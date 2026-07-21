use tokio::sync::RwLock;
use tower_lsp::{
    Client, LanguageServer, LspService, Server, jsonrpc::Result, lsp_types::*
};

struct Backend {
    client: Client,
    documents: RwLock<Vec<TextDocumentItem>>
}

impl Backend {
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(
                    TextDocumentSyncCapability::Kind(
                        TextDocumentSyncKind::FULL,
                    ),
                ),
                hover_provider: Some(
                    HoverProviderCapability::Simple(true)
                ),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "my-language-server".into(),
                version: Some("0.1.0".into()),
            }),
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
        Ok(())
    }

    async fn did_open(
        &self,
        params: DidOpenTextDocumentParams,
    ) {
        let doc = params.text_document;

        self.documents
            .write()
            .await
            .push(doc);
    }

    async fn hover(
        &self,
        _params: HoverParams,
    ) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(
                MarkedString::String(
                    "Hello from my LSP".into()
                )
            ),
            range: None,
        }))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::new(|client| Backend {
            client,
            documents: RwLock::new(Vec::new())
        });

    Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}