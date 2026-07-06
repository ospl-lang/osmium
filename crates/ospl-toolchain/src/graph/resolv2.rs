use std::{collections::HashMap, path::PathBuf};

use crate::{graph::{build::{CxxNode, Graph, ModuleNode, ModuleNodeMeta, Requirement}, resolv0::{RModRef, PkgRef}, resolv1::HighGraph}};

pub fn lower(high: HighGraph) -> Graph {
    let mut modules: HashMap<u32, ModuleNode> = HashMap::new();

    for (id, high_mod) in high.modules {
        let mut resolved_requires: Vec<Requirement> = Vec::new();

        for (required_as, req) in &high_mod.requires {
            let target = match req {
                RModRef::Local(name) => {
                    let key = (high_mod.pkg.clone(), name.clone());
                    let i = high.module_index.get(&key)
                        .unwrap_or_else(|| panic!("missing local module: {:?}", key));

                    Requirement {
                        id: *i,
                        ident: required_as.clone()
                    }
                },

                RModRef::Extern(pkg, name) => {
                    let thing = get_package_module_with_name(pkg, &*name, &high.module_index);

                    Requirement {
                        id: thing,
                        ident: required_as.clone(),
                    }
                }
            };

            resolved_requires.push(target);
        }

        let mut cxx_deps = Vec::new();
        for (name, cxx) in &high_mod.extensions {
            let hsh = crate::util::hash_file(&cxx.file).expect("failed to get hash for CXX Extension");

            // let mut so_file = PathBuf::from(BUILD_FOLDER);
            let mut so_file = PathBuf::from(".");
            so_file.push(hsh.to_hex().to_string());

            cxx_deps.push(CxxNode {
                o_file: so_file,
                c_file: cxx.file.clone(),
                required_as: name.to_string(),
                link_with: cxx.link_with.clone()
            });
        }

        // Log!(Resolved, "{:?}", high_mod.ast);

        modules.insert(id, ModuleNode {
            // name: high_mod.name,
            ast: high_mod.ast,
            cxx_deps,
            deps: resolved_requires,
            meta: ModuleNodeMeta {
                name: high_mod.name,
                pkg: high_mod.pkgname.unwrap_or_else(|| "???".to_string()),
            }
        });
    }

    Graph {
        modules,
        main: high.main,
    }
}


// ugly but I've HAD ENOUGH of refactors so I'll just make shit slow
#[allow(unused)]
pub fn get_package_modules(
    pkg: &PkgRef,
    module_index: &HashMap<(PkgRef, String), u32>,
) -> Vec<u32> {
    module_index
        .iter()
        .filter_map(|((p, _name), id)| {
            if p == pkg {
                Some(*id)
            } else {
                None
            }
        })
        .collect()
}

pub fn get_package_module_with_name(
    pkg: &PkgRef,
    name: &str,
    module_index: &HashMap<(PkgRef, String), u32>,
) -> u32 {
    module_index
        .iter()
        .filter_map(|((p, n), id)| {
            if p == pkg && name == n {
                // Log!(Resolved, "module {n} in package {p:?}");
                Some(*id)
            } else {
                None
            }
        })
        .nth(0).expect("this package is not executable")
}
