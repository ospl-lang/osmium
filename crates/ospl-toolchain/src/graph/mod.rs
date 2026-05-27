use std::path::Path;

use ospl_common::ast::{Expr, Expression, FunctionType, FunctionValue, Literal, Position, Statement, Stmt, Type, decl::{Declaration, DeclarationMeta}};

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
                inner: Box::new(Expr::Call(
                    Expression {
                        at: Position::default(),
                        inner: Box::new(Expr::Literal(Literal::Function(FunctionValue {
                            block: v,
                            args: Vec::new(),
                            captures: Vec::new(),
                            ftype: FunctionType {
                                args: Vec::new(),
                                ret: Type::AnyScope,  // compiler infer
                            },
                        }))),
                    },
                    Vec::new(),
                ))
            },
            ty: None,
        }))
    }
}

pub fn create_ffi(name: &str, file: &Path) -> Statement {
    return Statement {
        at: Position::default(),
        inner: Box::new(Stmt::Define(Declaration {
            meta: DeclarationMeta { visibility: ospl_common::ast::decl::Visibility::Private, constness: ospl_common::ast::decl::Constness::Const },
            name: name.to_string(),
            rhs: Expression {
                at: Position::default(),
                inner: Box::new(Expr::FFILoad(Expression {
                    at: Position::default(),
                    inner: Box::new(Expr::Literal(Literal::Str(file.to_string_lossy().to_string()))),
                }))
            },
            ty: None
        }))
    }
}
