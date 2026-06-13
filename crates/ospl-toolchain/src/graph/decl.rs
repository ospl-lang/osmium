use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::graph::{build::UnresolvedRequirement, resolv0::{FinalPackageSetup, ModDef, ModSrc, RModRef, VersionDefinition}, resolv1::CExt};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UModRef {
    /// A module within the current package
    /// 
    /// - **0:** the package to include
    Local(String),

    /// A module within another package
    /// 
    /// - **0:** the other package's requirement name in the current package
    /// - **1:** the name of the module within that package
    Extern(String, String)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UModDef {
    pub at: ModSrc,

    #[serde(default)]
    pub require: HashMap<String, UModRef>,

    #[serde(default)]
    pub extensions: HashMap<String, CExt>,
}

#[derive(Default)]
pub struct PackageSetup {
    pub entry: String,
    pub name: Option<String>,
    pub version: HashMap<String, VersionDefinition>,

    pub includes: HashMap<String, UModDef>,
    pub requires: HashMap<String, UnresolvedRequirement>,
}

impl PackageSetup {
    pub fn finalize(self) -> FinalPackageSetup {
        let mut requires = Vec::new();
        let mut includes = HashMap::new();
        for (_, req) in self.requires.iter() {
            requires.push(req.clone());
        }

        for (mname, data) in self.includes {
            let mut require = HashMap::new();
            for (rname, req) in data.require {
                require.insert(rname.clone(), match req {
                    UModRef::Extern(e, l) => RModRef::Extern(
                        self.requires.get(&e).cloned().unwrap_or_else(|| {
                            panic!(
                                "failed to find {e} while resolving requirement {rname} in module {mname} in package {}",
                                self.name.clone().unwrap_or_else(|| "[unknown]".to_string()).clone(),
                            );
                        }).re,
                        l
                    ),
                    UModRef::Local(l) => RModRef::Local(l),
                });
            }
            includes.insert(mname, ModDef {
                at: data.at,
                extensions: data.extensions,
                require
            });
        }

        return FinalPackageSetup {
            entry: self.entry,
            name: self.name,
            version: self.version,
            requires,
            includes,
        }
    }
}