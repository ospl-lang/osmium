use crate::{VM, Value, inst::optimized::{InstBuilder, Opc}, tests::run_ast};

#[test]
pub fn loops() {
    let mut vm = VM::new();
    let insts = vec![
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

    vm.run_all(&insts);
}
