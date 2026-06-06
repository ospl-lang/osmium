use crate::{VM, function};
use ospl_common::inst::{RuntimeFunction, RuntimeValue, make};
use ::ospl_common::inst::optimized::{Inst, InstBuilder, Opc};

#[test]
/// Tests that function return values and arguments work
pub fn functions() {
    let mut vm = VM::new();
    vm.run_all(&vec![
        InstBuilder::new()
            .opcode(Opc::PushLiteral)
            .value(make::func(RuntimeFunction {
                captures: Vec::new(),
                code: vec![
                    InstBuilder::new()
                        .opcode(Opc::Add)
                        .index(0)
                        .index(1)
                        .build(),  // 2

                    InstBuilder::new()
                        .opcode(Opc::Ret)
                        .index(2)
                        .build()
                ]
            }))
            .build(),

        InstBuilder::new().opcode(Opc::PushLiteral)  // 1
            .value(make::int(5))
            .build(),

        InstBuilder::new().opcode(Opc::PushLiteral)  // 2
            .value(make::int(5))
            .build(),

        InstBuilder::new().opcode(Opc::Call)  // 3
            .index(0)
            .index(1)
            .index(2)
            .build(),
    ]);

    assert_eq!(vm.stack.frames.len(), 1, "should only be one frame after this call");
    assert_eq!(vm.get_value_top(3).as_bool(), Some(true), "5+5 should be 10");
}