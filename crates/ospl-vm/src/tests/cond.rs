use ospl_common::inst::optimized::{InstBuilder, Opc};

use crate::{VM, RuntimeValue};

#[test]
pub fn loops() {
    let mut vm = VM::new();
    let insts = vec![
        InstBuilder::new().opcode(Opc::PushLiteral).value(RuntimeValue::Int(0)).build(),  // 0
        InstBuilder::new().opcode(Opc::PushLiteral).value(RuntimeValue::Int(1)).build(),  // 1
        InstBuilder::new().opcode(Opc::PushLiteral).value(RuntimeValue::Int(10)).build(),  // 2
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

// #[test]
// pub fn ifs() {
//     let mut vm = VM::new();
//     let insts = vec![
//         InstBuilder::new()
//             .opcode(Opc::PushLiteral)
//             .value(RuntimeValue::Bool(true))
//             .build(),  // 0

//         InstBuilder::new()
//             .opcode(Opc::PushLiteral)
//             .value(RuntimeValue::Int(0))  // we want this to be 5
//             .build(),  // 1

//         InstBuilder::new()
//             .opcode(Opc::If)
//             .index(0)

//             // TRUE: WHAT WE WANT
//             .child(vec![
//                 InstBuilder::new()
//                     .opcode(Opc::PushLiteral)
//                     .value(RuntimeValue::Int(5))
//                     .build(),  // 2

//                 InstBuilder::new()
//                     .opcode(Opc::AssignRef)
//                     .index(1)
//                     .index(2)
//                     .build(),
//             ])

//             // FALSE: WHAT WE HATE
//             .child(vec![
//                 InstBuilder::new()
//                     .opcode(Opc::PushLiteral)
//                     .value(RuntimeValue::Int(4))
//                     .build(),  // 2

//                 InstBuilder::new()
//                     .opcode(Opc::AssignRef)
//                     .index(1)
//                     .index(2)
//                     .build(),
//             ])
//             .build()
//     ];

//     vm.run_all(&insts);
//     assert_eq!(*vm.get_value_top(1), RuntimeValue::Int(5));
//     assert_ne!(*vm.get_value_top(1), RuntimeValue::Int(4));
// }
