use std::{collections::HashMap, path::PathBuf};

use kdl::{KdlDocument, KdlValue};

use crate::graph::{build::UnresolvedRequirement, decl::{PackageSetup, UModDef, UModRef}, resolv0::{ModSrc, PkgRef, VersionRuleRef, VersionSelector}, resolv1::CExt};

pub fn parse_kdl(k: KdlDocument) -> Option<PackageSetup> {
    let mut p = PackageSetup::default();
    for root_node in k.nodes() {
        let n = root_node.name().value();

        if n == "entry" {
            let possibly_string = root_node.get(0)?.as_string()?;
            p.entry = possibly_string.to_string();
        }

        else if n == "name" {
            let possibly_string = root_node.get(0)?.as_string()?;
            p.name = Some(possibly_string.to_string());
        }

        else if n == "module" {
            let module_name: String = root_node.get(0)?.as_string()?.to_string();
            let mut src: Option<ModSrc> = None;
            let mut requirements = HashMap::new();
            let mut extensions = HashMap::new();

            let children = root_node.children()?;
            for child in children.nodes() {
                let n = child.name().value();
                if n == "file" {
                    let path = child.get(0)?.as_string()?.to_string();
                    src = Some(ModSrc::File(path));
                }

                if n == "import" {
                    let ident = child.get(0)?.as_string()?.to_string();
                    let children = child.children()?;
                    let child = children.get("from")?;
                    let access = child.get(0)?.as_string()?;
                    match access {
                        "local" => {
                            let get = child.get(1)?.as_string()?.to_string();
                            requirements.insert(ident.clone(), UModRef::Local(get));
                        },
                        "extern" => {
                            let from = child.get(1)?.as_string()?.to_string();
                            let get = child.get(2)?.as_string()?.to_string();
                            requirements.insert(ident.clone(), UModRef::Extern(from, get));
                        }
                        other => unimplemented!("{other}")
                    }
                }

                if n == "extension" {
                    let name = child.get(0)?.as_string()?.to_string();
                    let c_file = child.get(1)?.as_string()?.to_string();
                    let mut link_with = Vec::new();

                    if let Some(children) = child.children() {
                        for node in children.nodes() {
                            let n = node.name().value();
                            if n == "link" {
                                let lib_name = node.get(0)?;
                                link_with.push(lib_name.as_string()?.to_string());
                            }
                        }
                    }

                    extensions.insert(name, CExt {
                        file: PathBuf::from(c_file),
                        link_with
                    });
                }
            }

            p.includes.insert(module_name, UModDef {
                at: src?,
                require: requirements,
                extensions,
            });
        }

        else if n == "requirement" {
            let requirement_name = root_node.get(0)?.as_string()?.to_string();
            let children = root_node.children()?;

            // parse rule field
            let requirement_ref_child = children.get("from")?;
            let requirement_ref = match requirement_ref_child.get(0)?.as_string()? {
                "git" => {
                    let repo = children.get("repo")?.get(0)?.as_string()?.to_string();

                    PkgRef::Git {
                        repo,
                    }
                },
                x => unimplemented!("{x:?}")
            };

            // parse version field
            let child = children.get("version")?;
            let requirement_version_selector_name = child.get(0)?.as_string()?;
            let requirement_version_selector = child.get(1)?;

            // create requirement
            let requirement = UnresolvedRequirement {
                re: requirement_ref,
                ver: VersionRuleRef {
                    name: requirement_version_selector_name.to_string(),
                    selection: match requirement_version_selector {
                        KdlValue::Integer(i) => VersionSelector::Int(*i as u64),
                        KdlValue::Float(f) => VersionSelector::Float(*f),
                        KdlValue::Bool(b) => VersionSelector::Flag(*b),
                        _ => return None
                    }
                }
            };

            p.requires.insert(requirement_name, requirement);
        }
    }

    return Some(p)
}
