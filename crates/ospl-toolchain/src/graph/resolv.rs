use std::{collections::HashMap, fmt::Debug, path::{Path, PathBuf}};
// use std::path::PathBuf;
// use std::collections::HashSet;
use ospl_common::ast::Statement;

use crate::{Log, LogState, graph::decl::{ModRef, ModSrc, PackageSetup, PkgRef}, load_package_yml_at};

#[derive(Default, Debug)]
pub struct HighGraph {
    pub modules: HashMap<u32, HighModuleNode>,

    /// GPT says this means:
    /// > "Given a symbolic module reference, what ModuleId does that refer to?"
    pub module_index: HashMap<(PkgRef, String), u32>,

    pub main: u32,
    next_id: u32,
}

impl HighGraph {
    pub fn next(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        return id
    }
}

pub struct HighModuleNode {
    pub name: String,
    pub ast: Vec<Statement>,
    pub requires: HashMap<String, ModRef>,   // still unresolved

    /// Path to C extension file relative to current package
    pub extensions: HashMap<String, PathBuf>,

    pub pkg: PkgRef,
}

impl Debug for HighModuleNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{{ {} needs {:?} }}", self.name, self.requires);
    }
}

#[derive(Clone)]
pub struct RecursionInfo {
    pub pkg: PkgRef,
    current_folder: PathBuf,
}

impl Default for RecursionInfo {
    fn default() -> Self {
        return Self {
            current_folder: std::env::current_dir().unwrap(),
            pkg: PkgRef::default(),
        }
    }
}

pub fn resolve_pkg(p: PackageSetup, q: &mut HighGraph, mut re: RecursionInfo) {
    // --- build modules ---
    LogState!(&format!("{:?}", re.pkg));
    for (name, mdl) in &p.includes {
        Log!(Resolving, "module `{name}`");

        let key = (re.pkg.clone(), name.clone());

        // already exists -> skip
        if q.module_index.contains_key(&key) {
            continue;
        }

        let ast = match &mdl.at {
            ModSrc::File(f) => {
                re.current_folder.push(f);
                let code = parse_file(&re.current_folder);
                re.current_folder.pop();
                code
            }
        };

        let id = q.next_id;
        q.next_id += 1;

        q.module_index.insert(key, id);

        let mut absolute_cxx_paths = HashMap::new();
        for (include_as, path) in &mdl.extensions {
            let mut true_path = re.current_folder.clone();
            true_path.extend(path);
            absolute_cxx_paths.insert(include_as.clone(), true_path);
        }

        q.modules.insert(id, HighModuleNode {
            name: name.clone(),
            ast,
            requires: mdl.require.clone(),
            extensions: absolute_cxx_paths,
            pkg: re.pkg.clone()
        });
    }

    // --- recurse dependencies ---
    for req in &p.requires {
        Log!(Resolving, "requirement {req:?}");

        match req {
            PkgRef::Local(l) => {
                re.current_folder.push(l);
                let next_re = RecursionInfo {
                    pkg: req.clone(),
                    current_folder: re.current_folder.clone()
                };

                let mut pkg_yml = re.current_folder.clone();
                pkg_yml.push("package.yml");
                let p = load_package_yml_at(&pkg_yml)
                    .expect("failed to load package.yml");

                re.current_folder.pop();

                resolve_pkg(p, q, next_re);
            }
            _ => {}
        }
    }
}

pub fn parse_file<P: AsRef<Path>>(f: P) -> Vec<Statement> {
    let s = std::fs::read_to_string(f).expect("TODO unwrap");
    let mut l = ospl_parser::lexer::lexer::Lexer::new(&s);
    let tokens = l.all_tokens();

    let mut p = ospl_parser::parse::Parser::new(&tokens);
    let ast = p.parse_file().expect("TODO unwrap");
    return ast
}