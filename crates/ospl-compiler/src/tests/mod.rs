#![allow(unused_imports)]
use crate::Compiler;
use ospl_common::ast::repr::{FunctionData, StaticType, Stmt};
use ospl_common::inst::{RuntimeFunctionData, RuntimeStaticValue};
use ::ospl_common::inst::unoptimized::VMInstruction;
use ::ospl_common::ast::repr::{Expr, LValue, Op, StaticValue};

mod integration;

#[test]
fn declare_and_add() {
    let mut comp = Compiler::default();
    let mut root = Vec::new();
    comp.declare_lvalue_init(LValue::Var("x".to_string()), Expr::StaticLiteral(StaticValue::Int(9)), &mut root).expect("error");
    comp.declare_lvalue_init(LValue::Var("y".to_string()), Expr::StaticLiteral(StaticValue::Int(10)), &mut root).expect("error");
    comp.declare_lvalue_init(LValue::Var("z".to_string()), Expr::BinaryOp {
        op: Op::Add,
        lhs: Box::new(Expr::LValue(LValue::Var("x".to_string()))),
        rhs: Box::new(Expr::LValue(LValue::Var("y".to_string())))
    }, &mut root).expect("error");

    assert_eq!(
        root,
        vec![
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(9)),
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(10)),
            VMInstruction::AddRegs(0, 1),
        ]
    );
}

#[test]
fn assign() {
    let mut comp = Compiler::default();
    let mut root = Vec::new();
    comp.declare_lvalue_init(LValue::Var("x".to_string()), Expr::StaticLiteral(StaticValue::Int(1)), &mut root).expect("error");
    comp.declare_lvalue_init(LValue::Var("y".to_string()), Expr::StaticLiteral(StaticValue::Int(3)), &mut root).expect("error");
    comp.assign_lvalue(LValue::Var("x".to_string()), &Expr::LValue(LValue::Var("y".to_string())), &mut root).expect("error");

    assert_eq!(
        root,
        vec![
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(1)),  // 0
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(3)),  // 1
            VMInstruction::AssignCopy {
                reg: 0,
                new: 1
            }
        ]
    );
}

#[test]
fn function() {
    let mut comp = Compiler::default();
    let mut root = Vec::new();
    comp.declare_lvalue_init(LValue::Var("add".to_string()), Expr::StaticLiteral(StaticValue::Function(FunctionData {
        code: vec![
            Stmt::Return(Expr::StaticLiteral(StaticValue::Int(42)))
        ],
        args: vec![StaticType::Int, StaticType::Int],
        ret: StaticType::Int
    })), &mut root).expect("error");

    comp.declare_lvalue_init(LValue::Var("x".to_string()), Expr::Call(
        Box::new(Expr::LValue(LValue::Var("add".to_string()))),
        vec![Expr::StaticLiteral(StaticValue::Int(9)), Expr::StaticLiteral(StaticValue::Int(10))]
    ), &mut root).expect("error");

    assert_eq!(
        root,
        vec![
            VMInstruction::PushLiteral(RuntimeStaticValue::Function(RuntimeFunctionData {
                code: vec![
                    VMInstruction::PushLiteral(RuntimeStaticValue::Int(42)),  // 0
                    VMInstruction::Ret(0)
                ]
            }))
        ]
    )
}
