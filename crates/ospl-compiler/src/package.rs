use ospl_common::ast::{Statement, decl::Declaration};
use crate::EvalResult;

#[derive(Debug, Default)]
pub struct Packages {
    pub modules: std::collections::HashMap<String, Module>,
}

#[derive(Debug, Default)]
pub struct Module {
    pub exports: Vec<Export>,
    pub ast: Vec<Statement>,
}

#[derive(Debug)]
pub struct Export {
    pub decl: Declaration,
    pub cached: Option<EvalResult>,
}
