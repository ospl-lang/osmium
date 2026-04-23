use std::collections::HashMap;

use crate::Compiler;
use ::ospl_common::ast::class::Definition;
use ::ospl_common::ast::class::builder::CTableBuilder;
use ospl_common::ast::class::instance::{InstanceItem, InstanceType};
use ::ospl_common::ast::module::VPath;
use ::ospl_common::ast::repr::{FunctionData, FunctionType, Parameter, Stmt, Type};
use ::ospl_common::inst::{RuntimeFunctionData, RuntimeStaticValue};
use ::ospl_common::inst::unoptimized::VMInstruction;
use ::ospl_common::ast::repr::{Expr, LValue, StaticValue};

mod decl;

#[test]
fn function() {
    let mut comp = Compiler::new();
    let mut root = Vec::new();
    comp.declare_real(&LValue::Var("add".to_string()), &Expr::StaticLiteral(StaticValue::Function(FunctionData {
        code: vec![
            Stmt::Return(Expr::StaticLiteral(StaticValue::Int(42)))
        ],
        ty: FunctionType {
            args: vec![
                Parameter {
                    ident: "x".to_string(),
                    ty: Type::Int,
                },
                Parameter {
                    ident: "y".to_string(),
                    ty: Type::Int,
                },
            ],
            ret: Box::new(Type::Int)
        }
    })), &mut root).expect("error");

    comp.declare_real(&LValue::Var("x".to_string()), &Expr::Call(
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
            })),  // 0
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(9)),  // 1
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(10)),  // 2
            VMInstruction::Call(0, vec![1, 2])  // 3
        ]
    )
}

#[test]
fn vscope() {
    let mut comp = Compiler::new();
    let vp = VPath::from("ExampleClass".to_string());
    let orig = CTableBuilder::default()
        .build();

    comp.declare_class(&vp, &orig)
        .expect("declare class failed");

    let Ok((cont, last)) = comp.vroot.get_container_mut(&vp)
        else { panic!("vpath error (possibly an issue with VPath::create_member)") };

    let cont = &*cont;
    let l = &cont.members[last];
    assert_eq!(*l, Type::Class(orig));
}

#[test]
fn class_construct() {
    let mut root = Vec::new();
    let mut comp = Compiler::new();

    // create the class
    let f = StaticValue::Function(FunctionData {
        code: vec![
            Stmt::Return(Expr::StaticLiteral(StaticValue::Int(1)))
        ],
        ty: FunctionType {
            args: vec![],
            ret: Box::new(Type::Int)
        }
    });

    let cls = CTableBuilder::default()
        .define("test".to_string(), Definition {
            ty: f.clone().into(),
            val: f.clone()
        })  // 0 in instance
        .build();  // 0 in instance's parent

    // instantiate it
    let e = comp.do_instantiate_class(&cls, &mut root)
        .expect("failed to eval instantiation");

    assert_eq!(e.index, 0, "instance's index should be 0");

    let mut expect = HashMap::new();
    expect.insert("test".to_string(), InstanceItem {
        ty: Box::new(Type::Function(FunctionType { args: vec![], ret: Box::new(Type::Int) })),
        index: 0
    });
    
    assert_eq!(e.ty, Type::Instance(InstanceType {
        values: expect,
    }), "instance's type is wrong");
}
