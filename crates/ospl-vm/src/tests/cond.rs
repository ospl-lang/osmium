use ospl_common::inst::{make, optimized::{Inst, InstBuilder, Opc}};

use crate::{RuntimeValue, VM, tests};

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
            .build(),
    ];
    return insts;
}

#[test]
pub fn loops_inside_functions() {
    let mut vm = VM::new();

    let mut fcode: Vec<Inst> = loops_code(10);
    fcode.push(InstBuilder::new().opcode(Opc::Ret).index(0).build());

    let mut code = vec![
        InstBuilder::new().opcode(Opc::PushFunction)
            .child(fcode)
            .build(), // 0

        InstBuilder::new().opcode(Opc::Call).index(0).build()  // 1
    ];
    println!("THE CODE HERE IT IS {code:?}");
    vm.run_all(&code);
}

#[test]
pub fn loops() {
    let mut vm = VM::new();
    vm.run_all(&loops_code(10));
    assert_eq!(*vm.get_value_top(0), make::int(10));
    assert_eq!(*vm.get_value_top(1), make::int(1));
    assert_eq!(*vm.get_value_top(2), make::int(10));
    println!("{vm:?}");
}
