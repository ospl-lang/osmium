// required in tests because rust-analyzer is a piece of shit
#![allow(unused_imports)]
#![allow(unused)]
use ospl_common::inst::optimized::{Inst, InstBuilder, Opc};

use crate::{VM, RuntimeValue, function};

#[test]
pub fn ifs() {
    let mut vm = VM::new();
    vm.run_all(&vec![
        InstBuilder::new().opcode(Opc::PushLiteral).value(RuntimeValue::Int(10)).build(),  // 0
        InstBuilder::new().opcode(Opc::PushLiteral).value(RuntimeValue::Int(9)).build(),   // 1
        InstBuilder::new().opcode(Opc::Eq)  // 2
            .index(0)
            .index(1)
            .build(),

        InstBuilder::new().opcode(Opc::PushLiteral).value(RuntimeValue::Undefined).build(),   // 3
        InstBuilder::new().opcode(Opc::If)
            .index(2)
            .child(vec![
                InstBuilder::new().opcode(Opc::AssignLiteral)
                    .index(3)
                    .value(RuntimeValue::Bool(true))
                    .build()
            ])
            .child(vec![
                InstBuilder::new().opcode(Opc::AssignLiteral)
                    .index(3)
                    .value(RuntimeValue::Bool(false))
                    .build()
            ])
            .build()
    ]);

    assert_eq!(vm.get_value_top(3).as_bool(), Some(false), "9 != 10");
}

mod funcs;
mod cond;

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
