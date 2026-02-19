// required in tests because rust-analyzer is a piece of shit
#![allow(unused_imports)]
#![allow(unused)]
use crate::{VM, Value, function, inst::optimized::{Inst, InstBuilder, Opc}};

#[test]
/// Tests that function return values and arguments work
pub fn functions() {
    let mut vm = VM::new();
    vm.run_all(&vec![
        InstBuilder::new()
            .opcode(Opc::PushLiteral)
            .value(Value::Fn(function::Fn::new_literal(Vec::new(), vec![
                InstBuilder::new()
                    .opcode(Opc::Add)
                    .index(0)
                    .index(1)
                    .build(),  // 2

                InstBuilder::new()
                    .opcode(Opc::Ret)
                    .index(2)
                    .build()
            ])))
            .build(),

        InstBuilder::new().opcode(Opc::PushLiteral)  // 1
            .value(Value::Int(5))
            .build(),

        InstBuilder::new().opcode(Opc::PushLiteral)  // 2
            .value(Value::Int(5))
            .build(),

        InstBuilder::new().opcode(Opc::Call)  // 3
            .index(0)
            .index(1)
            .index(2)
            .build(),
    ]);

    assert_eq!(vm.stack.len(), 1, "should only be one frame after this call");
    assert_eq!(vm.get_value_top(3).as_bool(), Some(true), "5+5 should be 10");
}

#[test]
pub fn ifs() {
    let mut vm = VM::new();
    vm.run_all(&vec![
        InstBuilder::new().opcode(Opc::PushLiteral).value(Value::Int(10)).build(),  // 0
        InstBuilder::new().opcode(Opc::PushLiteral).value(Value::Int(9)).build(),   // 1
        InstBuilder::new().opcode(Opc::Eq)  // 2
            .index(0)
            .index(1)
            .build(),

        InstBuilder::new().opcode(Opc::PushLiteral).value(Value::Undefined).build(),   // 3
        InstBuilder::new().opcode(Opc::If)
            .index(2)
            .child(vec![
                InstBuilder::new().opcode(Opc::AssignLiteral)
                    .index(3)
                    .value(Value::Bool(true))
                    .build()
            ])
            .child(vec![
                InstBuilder::new().opcode(Opc::AssignLiteral)
                    .index(3)
                    .value(Value::Bool(false))
                    .build()
            ])
            .build()
    ]);

    assert_eq!(vm.get_value_top(3).as_bool(), Some(false), "9 != 10");
}

#[test]
fn loops() {
    let mut vm = VM::new();
    let code = vec![
        InstBuilder::new().opcode(Opc::PushLiteral).value(Value::Int(0)).build(),  // 0
        InstBuilder::new().opcode(Opc::PushLiteral).value(Value::Int(1)).build(),  // 1
        InstBuilder::new().opcode(Opc::PushLiteral).value(Value::Int(10)).build(),  // 2
        InstBuilder::new()
            .opcode(Opc::Loop)
            .child(vec![
                InstBuilder::new()
                    .opcode(Opc::Addl)
                    .index(0)
                    .index(1)
                    .build(),

                InstBuilder::new()
                    .opcode(Opc::Eq)
                    .index(0)
                    .index(2)
                    .build(),  // 3

                InstBuilder::new()
                    .opcode(Opc::If)
                    .index(3)
                    .child(vec![
                        InstBuilder::new().opcode(Opc::Break).build()
                    ])
                    .child(Vec::new())
                    .build()
            ])
            .build()
    ];

    run_ast("loops", &code);
}

pub fn run_ast(f: &str, insts: &[Inst]) {
    let mut vm: VM = VM::new();

    use ::std::time::Instant;

    let start = Instant::now();

    vm.run_all(insts);

    let end = Instant::now();
    let wc = end - start;

    let mut modname = module_path!();
    println!("test {modname}::{} ... {:?}", f, wc);
}
