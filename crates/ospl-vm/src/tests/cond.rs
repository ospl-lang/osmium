use ospl_common::inst::{make, optimized::{Inst, InstBuilder, Opc}};

use crate::{VM, RuntimeValue};

pub fn loops_code(x:i64) -> Vec<Inst> {
    let insts = vec![
        InstBuilder::new().opcode(Opc::PushLiteral).value(make::int(0)).build(),  // 0
        InstBuilder::new().opcode(Opc::PushLiteral).value(make::int(1)).build(),  // 1
        InstBuilder::new().opcode(Opc::PushLiteral).value(make::int(x)).build(),  // 2
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
    return insts;
}

#[test]
pub fn loops() {
    let mut vm = VM::new();
    vm.run_all(&loops_code(10));
}
