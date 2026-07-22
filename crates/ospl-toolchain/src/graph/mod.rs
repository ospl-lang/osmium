use std::{path::Path, sync::Arc};

use ospl_common::ast::{Expr, Expression, FunctionType, FunctionValue, Literal, Position, Statement, Stmt, UType, decl::Declaration};

pub mod decl;
pub mod resolv0;
pub mod resolv1;
pub mod resolv2;
pub mod build;

pub fn wrap_in_iife_declaration(name: &str, mut v: Vec<Statement>, file_path: Arc<String>) -> Statement {
    v.push(Statement {
        file: Arc::clone(&file_path),
        at: Position::default(),
        inner: Box::new(Stmt::ReturnScope),
        notes: String::new(),
    });

    return Statement {
        file: Arc::clone(&file_path),
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            name: name.to_string(),
            rhs: Expression {
                file: Arc::clone(&file_path),
                at: Position::default(),
                inner: Box::new(Expr::Call(
                    Expression {
                        file: Arc::clone(&file_path),
                        at: Position::default(),
                        inner: Box::new(Expr::Literal(Literal::Function(FunctionValue {
                            block: v,
                            args: Vec::new(),
                            captures: Vec::new(),
                            ftype: FunctionType {
                                args: Vec::new(),
                                ret: UType::InferScope
                            },
                        }))),
                    },
                    Vec::new(),
                ))
            },
        })),
        notes: String::new(),
    }
}

pub fn wrap_in_declaration(name: &str, mut v: Vec<Statement>, file_path: Arc<String>) -> Statement {
    v.push(Statement {
        file: Arc::clone(&file_path),
        at: Position::default(),
        inner: Box::new(Stmt::ReturnScope),
        notes: String::new(),
    });

    Statement {
        file: Arc::clone(&file_path),
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            name: name.to_string(),
            rhs: Expression {
                file: Arc::clone(&file_path),
                at: Position::default(),
                inner: Box::new(Expr::Literal(Literal::Function(FunctionValue {
                    block: v,
                    args: Vec::new(),
                    captures: Vec::new(),
                    ftype: FunctionType {
                        args: Vec::new(),
                        ret: UType::InferScope,
                    },
                }))),
            },
        })),
        notes: String::new(),
    }
}

pub fn create_ffi(name: &str, file: &Path) -> Statement {
    let f = Arc::new(file.to_string_lossy().to_string());
    return Statement {
        file: Arc::clone(&f),
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            name: name.to_string(),
            rhs: Expression {
                file: Arc::clone(&f),
                at: Position::default(),
                inner: Box::new(Expr::FFILoad(Expression {
                    file: Arc::clone(&f),
                    at: Position::default(),
                    inner: Box::new(Expr::Literal(Literal::Str(file.to_string_lossy().to_string()))),
                }))
            },
        })),
        notes: String::new(),
    }
}
