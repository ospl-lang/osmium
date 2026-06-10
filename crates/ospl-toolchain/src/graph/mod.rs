use std::path::Path;

use ospl_common::ast::{Expr, Expression, FunctionType, FunctionValue, Literal, Position, Statement, Stmt, UType, decl::Declaration};

pub mod decl;
pub mod resolv0;
pub mod resolv1;
pub mod resolv2;
pub mod build;

pub fn wrap_in_iife_declaration(name: &str, mut v: Vec<Statement>) -> Statement {
    v.push(Statement {
        at: Position::default(),
        inner: Box::new(Stmt::ReturnScope)
    });

    return Statement {
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            name: name.to_string(),
            rhs: Expression {
                at: Position::default(),
                inner: Box::new(Expr::Call(
                    Expression {
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
        }))
    }
}

pub fn create_ffi(name: &str, file: &Path) -> Statement {
    return Statement {
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            name: name.to_string(),
            rhs: Expression {
                at: Position::default(),
                inner: Box::new(Expr::FFILoad(Expression {
                    at: Position::default(),
                    inner: Box::new(Expr::Literal(Literal::Str(file.to_string_lossy().to_string()))),
                }))
            },
        }))
    }
}
