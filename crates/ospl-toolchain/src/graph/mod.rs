use std::path::Path;

use ospl_common::ast::{
    Expr, Expression, FunctionValue, Literal, Position, Statement, Stmt,
    decl::{Declaration, DeclarationMeta}, types::{FunctionType, UType}
};

pub mod resolv;
pub mod resolv2;
pub mod decl;
pub mod build;

pub fn wrap_in_iife_declaration(name: &str, mut v: Vec<Statement>) -> Statement {
    v.push(Statement {
        at: Position::default(),
        inner: Box::new(Stmt::ReturnScope)
    });

    return Statement {
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            meta: DeclarationMeta {
                constness: ospl_common::ast::decl::Constness::Const,
                visibility: ospl_common::ast::decl::Visibility::Private
            },
            name: name.to_string(),
            rhs: Expression {
                at: Position::default(),
                inner: Box::new(Expr::Call {
                    func: Expression {
                        at: Position::default(),
                        inner: Box::new(Expr::Literal(Literal::Function(FunctionValue {
                            block: v,
                            args: Vec::new(),
                            captures: Vec::new(),
                            ftype: FunctionType {
                                args: Vec::new(),
                                generics: Vec::new(),
                                ret: UType::AnyScope,  // compiler infer
                            },
                            generics: Vec::new()
                        }))),
                    },
                    args: Vec::new(),
                    generics: Vec::new(),
                })
            },
        }))
    }
}

pub fn create_ffi(name: &str, file: &Path) -> Statement {
    return Statement {
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            meta: DeclarationMeta {
                visibility: ospl_common::ast::decl::Visibility::Private,
                constness: ospl_common::ast::decl::Constness::Const
            },
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
