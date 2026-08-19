use std::{collections::HashMap, path::{Component, Path, PathBuf}};

use kdl::{KdlDocument, KdlNode, KdlValue};

use crate::graph::{build::UnresolvedRequirement, decl::{PackageSetup, UModDef, UModRef}, resolv0::{ModSrc, PkgRef, VersionDefinition, VersionRule, VersionRuleRef, VersionSelector, VersionTag}, resolv1::CExt};

/// Rejects any path that isn't relative to the package folder (i.e. absolute
/// paths, or paths that escape the package folder via `..`).
fn sanitize_relative_path(what: &str, p: &str) -> String {
    let path = Path::new(p);
    if path.is_absolute() {
        panic!("the {what} path `{p}` must be relative to the package folder");
    }

    for comp in path.components() {
        match comp {
            Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                panic!("the {what} path `{p}` must stay within the package folder");
            }
            _ => {}
        }
    }

    return p.to_string()
}

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
                    src = Some(ModSrc::File(sanitize_relative_path("module", &path)));
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
                        file: PathBuf::from(sanitize_relative_path("extension", &c_file)),
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
            let requirement_name = root_node
                .get(0)
                .expect("expected name for the `requirement` block.")
                .as_string()
                .expect("the name of a `requirement` block must be a string")
                .to_string();

            let children = root_node
                .children()
                .expect("expected children for the `requirement`");

            // parse rule field
            let requirement_ref_child = children.get("from").expect("`requirement` child block needs a `from`");
            let requirement_ref = match requirement_ref_child.get(0)?.as_string()? {
                "local" => {
                    let repo = requirement_ref_child.get(1)?.as_string()?.to_string();
                    PkgRef::Local(sanitize_relative_path("requirement", &repo))
                }
                "git" => {
                    let repo = requirement_ref_child.get(1)?.as_string()?.to_string();
                    PkgRef::Git(repo)
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
                        KdlValue::String(s) => VersionSelector::Str(s.clone()),
                        _ => return None
                    }
                }
            };

            p.requires.insert(requirement_name, requirement);
        }

        else if n == "versions" {
            fn parse_rule(n: &KdlNode) -> Option<VersionRule> {
                let selector_type = n.name().value();
                    // .expect(&format!("There must be a selector type\n{n}"))
                    // .expect(&format!("Selector types must be strings\n{n}"));

                let selector_data = n.get(0)?;

                let mut tags = Vec::new();
                for e in n.children()?.nodes().iter() {
                    let x = match e.name().value() {
                        "eol" => VersionTag::EOL,
                        "gone" => VersionTag::Gone,
                        "unmaintained" => VersionTag::Unmaintained,
                        "vulnerable" => VersionTag::Vulnerable(e.get(0)?.as_string()?.to_string()),
                        "error" => VersionTag::Error(e.get(0)?.as_string()?.to_string()),

                        "branch" => VersionTag::Branch(e.get(0)?.as_string()?.to_string()),
                        "commit" => VersionTag::Commit(e.get(0)?.as_string()?.to_string()),
                        "reload" => VersionTag::Reload(sanitize_relative_path("reload", &e.get(0)?.as_string()?.to_string())),
                        other => panic!("unknown version tag {other}")
                    };
                    tags.push(x);
                };

                return Some(VersionRule {
                    selector: match selector_type {
                        "int" => VersionSelector::Int(selector_data.as_integer()? as u64),
                        "float" => VersionSelector::Float(selector_data.as_float()? as f64),
                        "flag" => VersionSelector::Flag(selector_data.as_bool()?),
                        "str" => VersionSelector::Str(selector_data.as_string()?.to_string()),
                        other => panic!("unknown version selector type {other}")
                    },
                    rules: tags
                })
            }

            for child in root_node.children()?.nodes() {
                // we're parsing the `rules ...` block
                let mut v = VersionDefinition::default();
                if child.name().value() != "rules" { panic!("expected `rules`"); }
                let name = child.get(0)
                    .expect("expected name for a `rules` block")
                    .as_string()
                    .expect("the name of a `rules` block must be a string")
                    .to_string();

                for n in child.children()?.nodes() {
                    // we're parsing individual rules now
                    v.rules.push(parse_rule(n)?);
                }

                p.version.insert(name, v);
            }
        }
    }

    return Some(p)
}

#[cfg(test)]
mod tests {
    use super::sanitize_relative_path;

    #[test]
    fn accepts_relative_paths_within_package() {
        for (what, p) in [
            ("module", "main.ospl"),
            ("module", "src/lib.ospl"),
            ("extension", "ffi/ffi.c"),
            ("requirement", "."),
            ("requirement", "./vendor"),
            ("reload", "subdir/package.kdl"),
        ] {
            assert_eq!(sanitize_relative_path(what, p), p);
        }
    }

    #[test]
    #[should_panic(expected = "must be relative to the package folder")]
    fn rejects_absolute_paths() {
        sanitize_relative_path("module", "/etc/passwd");
    }

    #[test]
    #[should_panic(expected = "must stay within the package folder")]
    fn rejects_parent_traversal() {
        sanitize_relative_path("module", "../evil.ospl");
    }

    #[test]
    #[should_panic(expected = "must stay within the package folder")]
    fn rejects_deep_parent_traversal() {
        sanitize_relative_path("extension", "src/../../evil.c");
    }
}
