#![allow(unused_imports)]
use crate::Compiler;
use ::ospl_common::inst::VMInstruction;
use ::ospl_common::ast::repr::{Expr, LValue, Op, StaticValue};

#[test]
fn declare_and_add() {
    let mut comp = Compiler::default();
    comp.declare_lvalue_init(LValue::Var("x".to_string()), Expr::StaticLiteral(StaticValue::Int(9)));
    comp.declare_lvalue_init(LValue::Var("y".to_string()), Expr::StaticLiteral(StaticValue::Int(10)));
    comp.declare_lvalue_init(LValue::Var("z".to_string()), Expr::BinaryOp {
        op: Op::Add,
        lhs: Box::new(Expr::LValue(LValue::Var("x".to_string()))),
        rhs: Box::new(Expr::LValue(LValue::Var("y".to_string())))
    });

    assert_eq!(
        comp.code,
        vec![
            VMInstruction::PushLiteral(StaticValue::Int(9)),
            VMInstruction::PushLiteral(StaticValue::Int(10)),
            VMInstruction::AddRegs(0, 1),
        ]
    );
}

#[test]
fn assign() {
    let mut comp = Compiler::default();
    comp.declare_lvalue_init(LValue::Var("x".to_string()), Expr::StaticLiteral(StaticValue::Int(1)));
    comp.declare_lvalue_init(LValue::Var("y".to_string()), Expr::StaticLiteral(StaticValue::Int(3)));
    comp.assign_lvalue(LValue::Var("x".to_string()), &Expr::LValue(LValue::Var("y".to_string())));

    assert_eq!(
        comp.code,
        vec![
            VMInstruction::PushLiteral(StaticValue::Int(1)),  // 0
            VMInstruction::PushLiteral(StaticValue::Int(3)),  // 1
            VMInstruction::AssignCopy {
                reg: 0,
                new: 1
            }
        ]
    );
}
