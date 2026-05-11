use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CExtensionSetup {
    pub c: HashMap<String, CExtension>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CExtension {
    pub file: String,
}
