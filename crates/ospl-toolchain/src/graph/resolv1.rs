use std::{collections::HashMap, fmt::Debug, path::{Path, PathBuf}};
// use std::path::PathBuf;
// use std::collections::HashSet;
use ospl_common::ast::Statement;
use ospl_parser::parse::PE;
use serde::{Deserialize, Serialize};

use crate::{BUILD_FOLDER, Log, graph::{build::UnresolvedRequirement, resolv0::{FinalPackageSetup, ModSrc, PkgRef, RModRef, VersionRuleRef, VersionTag}}, load_package_cfg};

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
    pub extensions: HashMap<String, CExt>,

    pub pkg: PkgRef,
    pub pkgname: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct CExt {
    pub file: PathBuf,
    pub link_with: Vec<String>,
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

#[derive(Debug)]
pub enum ResolvErr {
    PE {
        file: String,
        err: PE
    }
}

#[must_use]
pub fn resolve_pkg(p: FinalPackageSetup, q: &mut HighGraph, re: RecursionInfo) -> Result<(), ResolvErr> {
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
                let code = parse_file(&p)?;

                code
            }
        };

        let id = q.next_id;
        q.next_id += 1;

        q.module_index.insert(key, id);

        let absolute_cinema: HashMap<String, CExt> = mdl.extensions.iter().map(|(_n, ce)| {
            let mut true_path = re.current_folder.clone();
            true_path.extend(&ce.file.clone());

            return (_n.clone(), CExt {
                file: true_path,
                ..ce.clone()
            })
        }).collect();

        q.modules.insert(id, HighModuleNode {
            pkgname: p.name.clone(),
            name: name.clone(),
            ast,
            requires: mdl.require.clone(),
            extensions: absolute_cinema,
            pkg: re.pkg.clone(),
        });
    }

    // --- recurse dependencies ---
    for rq in &p.requires {
        match &rq.re {
            PkgRef::Local(l) => do_folder(l, q, rq, &re)?,
            PkgRef::Git(repo) => {
                let hsh = blake3::hash(repo.as_bytes());
                let t = hsh.to_hex();
                let mut folder = PathBuf::from(BUILD_FOLDER);
                folder.push(t);

                // only download if it doesn't exist already
                if !PathBuf::from(&folder).exists() {
                    if !std::process::Command::new("git")
                        .arg("clone")
                        .arg("--quiet")
                        .arg("--depth")
                        .arg("1")
                        .arg(repo)
                        .arg(&folder)
                        .spawn().expect("failed to invoke git")
                        .wait().expect("failed to wait for git")
                        .success()
                    { panic!("git exited unsuccessfully"); };
                }

                // let old = std::env::current_dir().unwrap();
                // std::env::set_current_dir(&folder).unwrap();
                // do_folder(&".", q, rq, &re);
                do_folder(&folder, q, rq, &re)?;
                // std::env::set_current_dir(old).unwrap();
            }
        }
    }

    Ok(())
}

fn do_folder<P: AsRef<Path>>(l: &P, q: &mut HighGraph, rq: &UnresolvedRequirement, re: &RecursionInfo) -> Result<(), ResolvErr> {
    let mut pat = re.current_folder.clone();
    pat.push(l);
    let next_re = RecursionInfo {
        pkg: rq.clone().re,
        current_folder: pat.clone()
    };

    // get the package setup
    Log!(Entering, "{pat:?}");

    pat.push("package.kdl");
    let mut p = load_package_cfg(&pat).finalize();

    // we need to change the CWD into the current folder and back
    let old_wd = std::env::current_dir().unwrap();
    std::env::set_current_dir(next_re.current_folder.clone()).unwrap();
    let version_data = p.version.get(&rq.ver.name)
        .cloned()
        .unwrap_or_else(|| panic!("failed to get version {} of package {:?}. There exists: {:?}", rq.ver.name, p.name, p.version.keys()));

    // whether a `reload` rule asked us to re-read package.kdl after the
    // version rules (e.g. a git checkout) have been applied to the working tree
    let mut reload_with: Option<String> = None;

    for rule in &version_data.rules {
        if rule.selector == rq.ver.selection {
            for rule in &rule.rules {
                // this invokes `git` for various things, but it appears that it executes
                // in the directory the build process was started in.
                if let ApplyRuleToCwdCmd::ReloadKdl(file) = apply_rule_to_cwd(rule) {
                    reload_with = Some(file);
                }
                break;
            }
        }
    }

    // A git checkout (or similar) may have changed package.kdl, so re-read it
    // from the (possibly newly checked-out) folder before resolving it.
    if let Some(file) = reload_with {
        let mut reload_path = next_re.current_folder.clone();
        reload_path.push(&file);
        Log!(Reloading, "{reload_path:?}");
        p = load_package_cfg(&reload_path).finalize();
    }

    Log!(Leaving, "{old_wd:?}");


    // now we cd back out!
    std::env::set_current_dir(old_wd).unwrap();

    resolve_pkg(p, q, next_re)?;

    Ok(())
}

pub enum ApplyRuleToCwdCmd {
    Nothing,
    ReloadKdl(String)
}

/// Note that this function will permanently make it's environment unusable.
pub fn apply_rule_to_cwd(rule: &VersionTag) -> ApplyRuleToCwdCmd {
    match rule {
        // warns
        VersionTag::EOL => Log!(Warning, "this version is EOL!"),
        VersionTag::Unmaintained => Log!(Warning, "This version is unmaintained!"),
        VersionTag::Vulnerable(vuln) => Log!(Warning, "This version has a vulnerability: {vuln}"),

        // errors
        VersionTag::Gone => panic!("package version was removed"),
        VersionTag::Error(e) => panic!("the version blocked building, it claims: {e:?}"),

        // real tags
        VersionTag::Branch(b) => {
            if let Some(u) = has_uncommitted_changes() {
                panic!("refusing to abide by checkout / reset, uncommited changes\n{u}");
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
            if let Some(u) = has_uncommitted_changes() {
                panic!("refusing to abide by checkout / reset, uncommited changes!\n{u}");
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
        VersionTag::Reload(r) => {
            return ApplyRuleToCwdCmd::ReloadKdl(r.clone())
        }
    }

    return ApplyRuleToCwdCmd::Nothing
}

pub fn parse_file<P: AsRef<Path> + Debug>(f: P) -> Result<Vec<Statement>, ResolvErr> {
    let s = std::fs::read_to_string(&f).expect("TODO unwrap");
    let mut l = ospl_parser::lexer::lexer::Lexer::new(&s);
    let tokens = l.all_tokens();

    let fpath: String = f.as_ref().to_string_lossy().to_string();
    let mut p = ospl_parser::parse::Parser::new(&tokens, fpath.clone());
    return p.parse_tlc().map_err(|e| {  
        #[cfg(debug_assertions)]
        ospl_parser::parse::diag::print_diag(&p, &e);

        ResolvErr::PE {
            file: fpath.clone(),
            err: e
        }
    });
}

/// Helper to check if the CWD's git repo has uncommited changes
fn has_uncommitted_changes() -> Option<String> {
    Log!(Invoking, "git status --porcelain");
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("failed to check for uncommited git changes");

    if !output.status.success() || !output.stdout.is_empty() {
        return Some(String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        return None
    }
}