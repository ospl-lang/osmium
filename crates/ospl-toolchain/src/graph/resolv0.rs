use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::graph::{build::UnresolvedRequirement, resolv1::CExt};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PkgRef {
    Git {
        repo: String,
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
pub enum RModRef {
    Local(String),
    Extern(PkgRef, String)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModDef {
    pub at: ModSrc,

    #[serde(default)]
    pub require: HashMap<String, RModRef>,

    #[serde(default)]
    pub extensions: HashMap<String, CExt>,
}

#[derive(Clone, Debug)]
pub struct FinalPackageSetup {
    pub includes: HashMap<String, ModDef>,
    pub requires: Vec<UnresolvedRequirement>,

    pub entry: String,

    pub name: Option<String>,

    pub version: HashMap<String, VersionDefinition>,
}

#[derive(Clone, Debug)]
pub struct VersionDefinition {
    /// Rules before your rules
    pub prerules: Vec<VersionTag>,

    /// Your rules
    pub rules: Vec<VersionRule>
}

#[derive(Clone, Debug)]
pub struct VersionRule {
    /// The what
    pub selector: VersionSelector,

    /// The rules
    pub rules: Vec<VersionTag>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VersionSelector {
    Int(u64),
    Float(f64),
    Flag(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VersionRuleRef {
    pub name: String,
    pub selection: VersionSelector
}

#[derive(Clone, Debug)]
pub enum VersionTag {
    /*************************************************************************/
    /***   SOURCE DATA  ******************************************************/
    /*************************************************************************/
        /// Uses data from the desired commit
        Commit(String),

        /// Switches to the desired branch
        Branch(String),

        /// Produces a simple error on the user's screen abou
        Error(String),

    /*************************************************************************/
    /***   MARKERS AND TAGS   ************************************************/
    /*************************************************************************/
        /// Warns the user that this version is reaching end-of-life
        EOL,

        /// Warns the user that the version is unmaintained
        Unmaintained,
        
        /// Indicate this version used to exist but has since been removed
        Gone,

        /// Indicates this version is vulnerabile or has other security problems
        Vulnerable(String),
}

pub fn default_entry() -> String {
    "main".to_string()
}