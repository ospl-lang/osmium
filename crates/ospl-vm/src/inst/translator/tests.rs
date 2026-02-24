#![allow(unused_imports)]

use ospl_common::inst::{RuntimeStaticValue, unoptimized::VMInstruction};

use crate::{Value, inst::{optimized::{InstBuilder, Opc}, translator::vm_instruction_to_optimized}};

#[test]
fn three_basic_instructions() {
    // PushLiteral
    let opt = vm_instruction_to_optimized(VMInstruction::PushLiteral(RuntimeStaticValue::Int(42)));
    assert_eq!(opt, InstBuilder::new().opcode(Opc::PushLiteral).value(Value::Int(42)).build());

    // AssignCopy
    let opt = vm_instruction_to_optimized(VMInstruction::AssignCopy {
        reg: 0,
        new: 1
    });
    assert_eq!(opt, InstBuilder::new().opcode(Opc::AssignCopy).index(0).index(1).build());

    // AddRegs
    let opt = vm_instruction_to_optimized(VMInstruction::AddRegs(0, 1));
    assert_eq!(opt, InstBuilder::new().opcode(Opc::Add).index(0).index(1).build());
}
