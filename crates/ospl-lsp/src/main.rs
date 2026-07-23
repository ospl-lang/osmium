use std::{collections::HashMap, sync::Arc};

pub mod simplify_error;
pub mod hover;

use ospl_compiler::CE;
use ospl_parser::parse::PE;
use ospl_toolchain::{graph::resolv1::{RecursionInfo, ResolvErr}, load_package_cfg};
use tokio::sync::RwLock;
use tower_lsp::{
    Client, LanguageServer, LspService, Server, jsonrpc::{self, Result}, lsp_types::*
};

#[derive(Debug)]
pub enum BackendError {
    CE(CE),
    RE(ResolvErr),
    Panicked,
}

#[derive(Hash, PartialEq, Eq)]
struct HoverKey {
    document: Arc<String>,
    line: usize,
    column: usize,
}

struct Backend {
    client: Client,
    rootdir: RwLock<Url>,
    documents: RwLock<Vec<TextDocumentItem>>,
    errors: RwLock<Vec<BackendError>>,
    documentation: RwLock<HashMap<HoverKey, String>>
}

impl Backend {
    pub async fn rebuild(&self) {
        self.errors.write().await.clear();
        let e = std::panic::catch_unwind(|| {
            // -snip-
            let root_pkg = load_package_cfg("package.kdl").finalize();

            let entry_name = root_pkg.entry.clone();

            // high-level
            let mut gg = ospl_toolchain::graph::resolv1::HighGraph::default();
            let pi = RecursionInfo::default();

            if let Err(e) = ospl_toolchain::graph::resolv1::resolve_pkg(root_pkg, &mut gg, pi.clone()) {
                return Err(BackendError::RE(e))
            }

            gg.main = ospl_toolchain::graph::resolv2::get_package_module_with_name(&pi.pkg, &entry_name, &gg.module_index);
            ospl_toolchain::LogState!(&"");

            // low-level
            let low = ospl_toolchain::graph::resolv2::lower(gg);

            // compile
            let out = ospl_toolchain::graph::build::genmods(&low);
            let out = ospl_toolchain::graph::build::buildmain(&low, out);
            out.map_err(|e| BackendError::CE(e))
        });

        match e {
            Ok(e) => {
                match e {
                    Err(be) => self.errors.write().await.push(be),
                    Ok(_) => {}  // don't care about VM instructions
                }
            }
            Err(_something_panicked) => {
                // smth panicked
                self.errors.write().await.push(BackendError::Panicked)
            },
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> Result<InitializeResult> {
        if let Some(root_uri) = params.root_uri {
            self.client
                .log_message(MessageType::INFO, format!("Root: {root_uri}"))
                .await;

            *self.rootdir.write().await = root_uri;
        } else {
            return Err(jsonrpc::Error::new(jsonrpc::ErrorCode::InvalidParams))
        }

        if let Err(_) = std::env::set_current_dir(self.rootdir.read().await.path()) {
            return Err(jsonrpc::Error::new(jsonrpc::ErrorCode::InvalidParams))
        }

        self.client
            .log_message(MessageType::INFO, format!("End of initialize"))
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(
        &self,
        _: InitializedParams,
    ) {
        self.client
            .log_message(
                MessageType::INFO,
                "LSP initialized!",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(
                MessageType::ERROR,
                "Goodbye (shutdown)",
            )
            .await;
        Ok(())
    }

    async fn did_open(
        &self,
        params: DidOpenTextDocumentParams,
    ) {
        let doc = params.text_document;

        self.client
            .log_message(
                MessageType::INFO,
                format!("File opened: {}", doc.uri.as_str()),
            )
            .await;

        self.documents
            .write()
            .await
            .push(doc);
    }

    async fn did_save(
        &self,
        params: DidSaveTextDocumentParams
    ) {
        let td = params.text_document;

        self.client
            .log_message(
                MessageType::INFO,
                format!("File saved: {}", td.uri.as_str()),
            )
            .await;

        self.rebuild().await;

        // also reindex
        {
            let mut the_docs = self.documentation.write().await;
            for doc in self.documents.read().await.iter() {
                let Ok(x) = doc.uri.to_file_path()
                else { continue; };

                let ast = match ospl_toolchain::graph::resolv1::parse_file(&x) {
                    Err(e) => {
                        self.errors.write().await.push(BackendError::RE(e));
                        continue;
                    }
                    Ok(ast) => ast,
                };

                for thing in ast {
                    if !thing.notes.is_empty() {
                        let Some((hp, hs)) = hover::gen_hover(&thing)
                        else { continue; };

                        eprintln!("{}@{} {}", thing.at.line, thing.at.column, thing.notes);

                        for r in thing.at.line..=hp.line {
                            for c in thing.at.column..=hp.column {
                                the_docs.insert(HoverKey {
                                    document: Arc::clone(&thing.file),
                                    line: r - 1,
                                    column: c - 1,
                                }, hs.clone());
                            }
                        }
                    }
                }
            }
        };

        // delete the diags
        for doc in self.documents.read().await.iter() {
            self.client.publish_diagnostics(doc.uri.clone(), Vec::new(), None).await;
        }

        // repub diags
        let mut diagnostics_per_file: HashMap<String, Vec<Diagnostic>> = HashMap::new();
        for err in self.errors.read().await.iter() {
            eprintln!("{err:?}");
            match err {
                BackendError::CE(ce) => {
                    let file = ce.at.original_file().to_string();
                    let span = ce.at.get_pos();
                    let d = Diagnostic {
                        severity: Some(DiagnosticSeverity::ERROR),
                        range: Range::new(
                            Position::new(span.line as u32, span.column as u32),
                            Position::new((span.line + 1) as u32, 0)
                        ),
                        message: format!("{}", simplify_error::simplify(ce)),
                        ..Default::default()
                    };

                    if let Some(diags) = diagnostics_per_file.get_mut(&file) {
                        diags.push(d);
                    } else {
                        diagnostics_per_file.insert(file, vec![d]);
                    }
                },
                BackendError::RE(re) => {
                    match re {
                        ResolvErr::PE { file, err } => {
                            let d = match err {
                                PE::Expected(e) => {
                                    let span = e.got.position();
                                    let d = Diagnostic {
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        range: Range::new(
                                            Position::new(span.line as u32, span.column as u32),
                                            Position::new((span.line + 1) as u32, 0)
                                        ),
                                        message: format!("\n\
                                            expected: {}\n\
                                            got:      {:?}
                                        ", e.expected, e.got.token()),
                                        ..Default::default()
                                    };

                                    d
                                },
                                PE::EOF => continue
                            };

                            if let Some(diags) = diagnostics_per_file.get_mut(file) {
                                diags.push(d);
                            } else {
                                diagnostics_per_file.insert(file.to_string(), vec![d]);
                            }
                        }
                    }
                }
                BackendError::Panicked => {}
            }
        }

        eprintln!("{diagnostics_per_file:?}");

        for (file, diags) in diagnostics_per_file {
            let Ok(url) = Url::from_file_path(file.clone())
            else {
                eprintln!("invalid URL from file path: {file}");
                continue;
            };

            self.client.publish_diagnostics(
                url,
                diags,
                None
            ).await;
        }
    }

    async fn hover(
        &self,
        _params: HoverParams,
    ) -> Result<Option<Hover>> {
        let a = Arc::new(_params.text_document_position_params.text_document.uri
                                        .to_file_path()
                                        .map_err(|_| jsonrpc::Error::new(jsonrpc::ErrorCode::InvalidParams))?
                                        .to_string_lossy()
                                        .to_string());

        let b = self.documentation.read().await;
        let hover = b.get(&HoverKey {
            line: _params.text_document_position_params.position.line as usize,
            column: _params.text_document_position_params.position.character as usize,
            document: Arc::clone(&a)
        });

        if let Some(hover) = hover {
            Ok(Some(Hover {
                contents: HoverContents::Array(vec![
                    MarkedString::String(hover.to_string())
                ]),
                range: None,
            }))
        } else {
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(
                    MarkedString::String(format!(
                        "you're hovering at: {}:{}:{}",
                        _params.text_document_position_params.position.line as usize,
                        _params.text_document_position_params.position.character as usize,
                        a
                    ))   
                ),
                range: None
            }))
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::new(|client| Backend {
            client,
            documents: RwLock::new(Vec::new()),
            rootdir: RwLock::new(Url::from_directory_path(std::env::current_dir().unwrap()).unwrap()),
            errors: RwLock::new(Vec::new()),
            documentation: RwLock::new(HashMap::new())
        });

    Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}