use ospl_common::ast::Statement;
use crate::EvalResult;

#[derive(Debug, Default)]
pub struct Packages {
    pub modules: std::collections::HashMap<String, Module>,
}

#[derive(Debug, Default)]
pub struct Module {
    pub code: Vec<Statement>,
    pub cached: Option<EvalResult>,
}
