//! Longer, integration tests
#![allow(unused_imports)]

use ospl_common::{ast::repr::{Expr, FunctionData, LValue, StaticType, StaticValue, Stmt}, inst::{RuntimeFunctionData, RuntimeStaticValue, unoptimized::VMInstruction}};

use crate::Compiler;

#[test]
pub fn functions_returning_functions() {
    let mut compiler = Compiler::default();
    let mut root = Vec::new();
    compiler.declare_lvalue_init(LValue::Var("f".to_string()), Expr::StaticLiteral(StaticValue::Function(FunctionData {
        code: vec![
            Stmt::Return(Expr::StaticLiteral(StaticValue::Function(FunctionData {
                code: vec![
                    Stmt::Return(Expr::StaticLiteral(StaticValue::Function(FunctionData {
                        code: vec![
                            Stmt::Return(Expr::StaticLiteral(StaticValue::Int(42)))
                        ],
                        args: Vec::new(),
                        ret: StaticType::Int
                    })))
                ],
                args: Vec::new(),
                ret: StaticType::Function
            })))
        ],
        args: Vec::new(),
        ret: StaticType::Function
    })), &mut root).expect("compile failed");

    assert_eq!(
        root,
        vec![
            VMInstruction::PushLiteral(RuntimeStaticValue::Function(RuntimeFunctionData {
                code: vec![
                    VMInstruction::PushLiteral(RuntimeStaticValue::Function(RuntimeFunctionData {
                        code: vec![
                            VMInstruction::PushLiteral(RuntimeStaticValue::Function(RuntimeFunctionData {
                                code: vec![
                                    VMInstruction::PushLiteral(RuntimeStaticValue::Int(42)),
                                    VMInstruction::Ret(0),
                                ]
                            })),
                            VMInstruction::Ret(0)
                        ]
                    })),
                    VMInstruction::Ret(0)
                ]
            }))
        ]
    )
}