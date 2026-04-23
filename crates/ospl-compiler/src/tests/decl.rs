use crate::Compiler;
use ospl_common::ast::class::Definition;
use ospl_common::ast::class::builder::CTableBuilder;
use ospl_common::ast::module::VPath;
use ospl_common::ast::repr::Type;
use ::ospl_common::inst::RuntimeStaticValue;
use ::ospl_common::inst::unoptimized::VMInstruction;
use ::ospl_common::ast::repr::{Expr, LValue, Op, StaticValue};

#[test]
fn declare_and_add() {
    let mut comp = Compiler::new();
    let mut root = Vec::new();
    comp.declare_real(&LValue::Var("x".to_string()), &Expr::StaticLiteral(StaticValue::Int(9)), &mut root).expect("error");
    comp.declare_real(&LValue::Var("y".to_string()), &Expr::StaticLiteral(StaticValue::Int(10)), &mut root).expect("error");
    comp.declare_real(&LValue::Var("z".to_string()), &Expr::BinaryOp {
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
    let mut comp = Compiler::new();
    let mut root = Vec::new();
    comp.declare_real(&LValue::Var("x".to_string()), &Expr::StaticLiteral(StaticValue::Int(1)), &mut root).expect("error");
    comp.declare_real(&LValue::Var("y".to_string()), &Expr::StaticLiteral(StaticValue::Int(3)), &mut root).expect("error");
    comp.assign_real(&LValue::Var("x".to_string()), &Expr::LValue(LValue::Var("y".to_string())), &mut root).expect("error");

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
fn assign2() {
    let mut comp = Compiler::new();
    let mut root = Vec::new();
    comp.declare_real(&LValue::Var("x".to_string()), &Expr::StaticLiteral(StaticValue::Int(1)), &mut root).expect("error");  // 0
    comp.declare_real(&LValue::Var("y".to_string()), &Expr::StaticLiteral(StaticValue::Int(3)), &mut root).expect("error");  // 1
    comp.assign_real(&LValue::Var("x".to_string()), &Expr::BinaryOp {
        op: Op::Add,
        lhs: Box::new(Expr::LValue(LValue::Var("x".to_string()))),
        rhs: Box::new(Expr::LValue(LValue::Var("y".to_string())))
    }, &mut root).expect("error");  // 2

    assert_eq!(
        root,
        vec![
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(1)),  // 0
            VMInstruction::PushLiteral(RuntimeStaticValue::Int(3)),  // 1
            VMInstruction::AddRegs(0, 1),  // 2
            VMInstruction::AssignCopy {
                reg: 0,
                new: 2
            }
        ]
    );
}

#[test]
fn assign_instance() {
    let mut comp = Compiler::new();
    let mut root = Vec::new();
    comp.declare_class(&VPath::from("Class".to_string()), &CTableBuilder::default()
        .define("x".to_string(), Definition {
            ty: Type::Int,
            val: StaticValue::Int(1)
        })
        .build()
    ).unwrap();

    comp.declare_real(
        &LValue::Var("v".to_string()),
        &Expr::Construct(VPath::from("Class".to_string())),
        &mut root
    ).unwrap();
    // comp.vp_instantiate_class(&VPath::from("Class".to_string()), &mut root).unwrap();

    comp.assign_real(&LValue::Property(Box::new(
        LValue::Var("v".to_string())
        ), "x".to_string()), &Expr::StaticLiteral(StaticValue::Int(42)),
        &mut root
    ).unwrap();

    assert_eq!(root, vec![
        VMInstruction::PushLiteral(RuntimeStaticValue::Int(1)),
        VMInstruction::Construct { props: vec![0] },
        VMInstruction::PushLiteral(RuntimeStaticValue::Int(42)),
        VMInstruction::AssignCopy { reg: 0, new: 1 },
    ])
}
