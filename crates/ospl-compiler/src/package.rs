use std::collections::HashMap;

use ospl_common::ast::Statement;
use crate::EvalResult;

#[derive(Debug, Default)]
pub struct Packages {
    pub modules: std::collections::HashMap<String, Module>,
    pub c_extensions: HashMap<String /* ID */, CompiledCExtension>,
}

#[derive(Debug, Default)]
pub struct CompiledCExtension {
    pub compiled_path: String,
}

#[derive(Debug, Default)]
pub struct Module {
    pub code: Vec<Statement>,
    pub cached: Option<EvalResult>,
}
