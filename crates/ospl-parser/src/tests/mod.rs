#![allow(unused_imports)]
use crate::Parser;
use ::ospl_common::ast::repr::{Expr, LValue, StaticValue, Stmt};

#[test]
pub fn definitions() {
    let instr = "def xyz = 10;";
    let mut p = Parser::new(instr);
    let x = p.parse_stmt();

    assert_eq!(x, Some(Stmt::DeclareReal {
        lhs: LValue::Var("xyz".to_string()),
        rhs: Some(Expr::StaticLiteral(StaticValue::Int(10))),
    }));
}

#[test]
pub fn assignments() {
    let instr = "xyz = 10;";
    let mut p = Parser::new(instr);
    let x = p.parse_stmt();

    assert_eq!(x, Some(Stmt::AssignReal {
        lhs: LValue::Var("xyz".to_string()),
        rhs: Expr::StaticLiteral(StaticValue::Int(10)),
    }));
}
