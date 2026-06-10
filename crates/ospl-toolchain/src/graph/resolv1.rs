use std::{collections::HashMap, fmt::Debug, path::{Path, PathBuf}};
// use std::path::PathBuf;
// use std::collections::HashSet;
use ospl_common::ast::Statement;

use crate::{Log, graph::resolv0::{FinalPackageSetup, ModSrc, PkgRef, RModRef, VersionRuleRef, VersionTag}, load_package_cfg};

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
    pub requires: HashMap<String, RModRef>,   // still unresolved

    /// Path to C extension file relative to current package
    pub extensions: HashMap<String, PathBuf>,

    pub pkg: PkgRef,
    pub pkgname: Option<String>,
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

#[derive(Clone)]
pub struct CallInfo {
    pub at_version: VersionRuleRef,
}

impl Default for RecursionInfo {
    fn default() -> Self {
        return Self {
            current_folder: std::env::current_dir().unwrap(),
            pkg: PkgRef::default(),
        }
    }
}

pub fn resolve_pkg(p: FinalPackageSetup, q: &mut HighGraph, re: RecursionInfo) {
    // --- build modules ---
    for (name, mdl) in &p.includes {
        let key = (re.pkg.clone(), name.clone());

        // already exists -> skip
        if q.module_index.contains_key(&key) {
            continue;
        }

        let ast = match &mdl.at {
            ModSrc::File(f) => {
                let mut p = re.current_folder.clone();
                p.push(f);
                let code = parse_file(&p);

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
            pkgname: p.name.clone(),
            name: name.clone(),
            ast,
            requires: mdl.require.clone(),
            extensions: absolute_cxx_paths,
            pkg: re.pkg.clone(),
        });
    }

    // --- recurse dependencies ---
    for req in &p.requires {
        match &req.re {
            PkgRef::Local(l) => {
                let mut pat = re.current_folder.clone();
                pat.push(l);
                let next_re = RecursionInfo {
                    pkg: req.clone().re,
                    current_folder: pat.clone()
                };

                // we need to change the CWD into the current folder and back
                let old_wd = std::env::current_dir().unwrap();
                std::env::set_current_dir(re.current_folder.clone()).unwrap();
                let version_data = p.version.get(&req.ver.name).expect("failed to load version metadata");
                for rule in &version_data.prerules {
                    apply_rule_to_cwd(rule);
                }

                for rule in &version_data.rules {
                    if rule.selector == req.ver.selection {
                        for rule in &rule.rules {
                            apply_rule_to_cwd(rule);
                            break;
                        }
                    }
                }

                // now we cd back out!
                std::env::set_current_dir(old_wd).unwrap();

                pat.push("package.yml");
                let p = load_package_cfg(&pat).finalize();

                resolve_pkg(p, q, next_re);
            }
            _ => {}
        }
    }
}

/// Note that this function will permanently make it's environment unusable.
pub fn apply_rule_to_cwd(rule: &VersionTag) {
    match rule {
        // warns
        VersionTag::EOL => Log!(Warning, "this package is EOL!"),
        VersionTag::Unmaintained => Log!(Warning, "This package is unmaintained!"),
        VersionTag::Vulnerable(vuln) => Log!(Warning, "This package has a vulnerability: {vuln}"),

        // errors
        VersionTag::Gone => panic!("package version was removed"),
        VersionTag::Error(e) => panic!("the package blocked building, it claims: {e:?}"),

        // real tags
        VersionTag::Branch(b) => {
            if has_uncommitted_changes() {
                panic!("refusing to abide by checkout / reset, uncommited changes!");
            }

            Log!(Invoking, "git checkout {b}");

            let output = std::process::Command::new("git")
                .arg("checkout")
                .arg(b)
                .output()
                .expect("failed to execute git (is it installed and in PATH?)");

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("git checkout failed, git's stderr follows\n===\n{stderr}");
            }
        },
        VersionTag::Commit(commit_id) => {
            if has_uncommitted_changes() {
                panic!("refusing to abide by checkout / reset, uncommited changes!");
            }

            Log!(Invoking, "git reset --hard {commit_id}");

            let output = std::process::Command::new("git")
                .arg("reset")
                .arg("--hard")
                .arg(commit_id)
                .output()
                .expect("failed to execute git (is it installed and in PATH?)");

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("git reset failed, git's stderr follows\n===\n{stderr}");
            }
        },
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

/// Helper to check if the CWD's git repo has uncommited changes
fn has_uncommitted_changes() -> bool {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("failed to check for uncommited git changes");

    !output.status.success() || !output.stdout.is_empty()
}