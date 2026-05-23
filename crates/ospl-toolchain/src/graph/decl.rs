use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PkgRef {
    Git {
        repo: String,
        branch: Option<String>,
        commit: Option<String>,
    },

    /// "trust me, I have a local copy of this"
    Local(String),
}

impl Default for PkgRef {
    fn default() -> Self {
        return Self::Local(".".to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModSrc {
    File(String),
    // -snip-
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModRef {
    /// A module within the current package
    /// 
    /// - **0:** the package to include
    Local(String),

    /// A module within another package
    /// 
    /// - **0:** the other package's requirement name in the current package
    /// - **1:** the name of the module within that package
    Extern(PkgRef, String)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModDef {
    pub at: ModSrc,
    pub require: HashMap<String, ModRef>,

    #[serde(default)]
    pub extensions: HashMap<String, PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageSetup {
    pub includes: HashMap<String, ModDef>,
    pub requires: Vec<PkgRef>,

    #[serde(default = "default_entry")]
    pub entry: String,

    pub name: Option<String>,

    pub version: String,
    // pub extensions: CExtensionSetup,
}

fn default_entry() -> String {
    "main".to_string()
}