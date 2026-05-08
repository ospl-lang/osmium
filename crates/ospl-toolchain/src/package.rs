use std::{collections::HashMap, fs};

use ospl_compiler::{ast::{Statement, Stmt, decl::Visibility}, package::{Export, Module, Packages}};
use ospl_parser::{lexer::lexer::Lexer, parse::Parser};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub enum PkgRef {
    // Subpackage
    Github {
        repo: String,
        by: String,
        commit: String,
    },

    // Subpackage
    Folder(String),

    // Module
    File(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageSetup {
    pub includes: HashMap<String, PkgRef>,
    pub display: Option<String>,
    pub version: String,
}

impl PackageSetup {
    pub fn gensrc(&self) -> Packages {
        // generate our span
        let display_name: String = match &self.display {
            Some(x) => x.clone(),
            None => std::env::current_dir()
                .unwrap().file_name()
                .unwrap().to_str()
                .unwrap().to_string(),
        };

        // let _span = info_span!("in", name=display_name, version=self.version);
        // let _enter = _span.enter();

        info!(name=display_name, version=self.version, "generating repo");

        let mut packages = Packages::default();
        for (name, pkg) in &self.includes {
            info!(name=%name, "compiling module");
            let mut exports: Vec<Export> = Vec::new();
            let code;
            match pkg {
                PkgRef::File(f) => {
                    let s = fs::read_to_string(&*f)
                        .expect("local file module declaration not found");

                    code = do_main_functionality(&*s, &mut exports);
                },
                PkgRef::Folder(f) => {
                    // save our CWD
                    let prev_dir = std::env::current_dir().expect("failed to get current directory");

                    // enter the new directory of the folder
                    let mut new_dir = prev_dir.clone();
                    new_dir.push(f);

                    std::env::set_current_dir(new_dir).expect("failed to change directories");

                    // do our payload here
                    let p_tmp = load_pkgsetup_from_folder(".");
                    let mut p_reg = p_tmp.gensrc();
                    let p_mod = p_reg.modules.remove("library")
                        .expect("failed to find library target");

                    code = p_mod.ast;

                    // go back to the previous working directory
                    std::env::set_current_dir(prev_dir).expect("failed to change directories");
                },
                other => unimplemented!("PkgSource {other:?}"),
            }

            packages.modules.insert(name.to_string(), Module { exports, ast: code });
        };

        return packages
    }

    pub fn get_lib(&mut self) -> Option<&PkgRef> {
        let lib = self.includes.get("library")?;
        return Some(lib)
    }
}

pub fn load_pkgsetup_from_folder(f: &str) -> PackageSetup {
    let path = format!("{f}/package.yml");
    let s = fs::read_to_string(path).expect("package.yml not found in package folder");
    let cfg: PackageSetup = yaml_serde::from_str(&s).expect("failed to read package.yml");
    return cfg
}

pub fn do_main_functionality(s: &str, exports: &mut Vec<Export>) -> Vec<Statement> {
    let mut l = Lexer::new(&s);
    let t = l.all_tokens();

    let mut p = Parser::new(&t);
    let p = p.parse_file().expect("failed to parse module");

    stmt_to_exports(p.as_slice(), exports);
    return p
}

pub fn stmt_to_exports(p: &[Statement], exports: &mut Vec<Export>) {
    for stmt in p {
        match &*stmt.inner {
            Stmt::Define(dcl) => {
                if dcl.meta.visibility != Visibility::Public { continue }

                exports.push(Export {
                    cached: None,
                    decl: dcl.clone(),
                });
            },
            _ => {}
        }
    }
}