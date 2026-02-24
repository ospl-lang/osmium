use ospl_common::inst::{RuntimeStaticValue, unoptimized::VMInstruction};

use crate::{Value, inst::optimized::{Inst, InstBuilder, Opc}};

pub fn runtime_static_value_to_vm(v: RuntimeStaticValue) -> Value {
    return match v {
        RuntimeStaticValue::Int(i) => Value::Int(i),
        RuntimeStaticValue::Float(f) => Value::Float(f),
        RuntimeStaticValue::Nul => Value::Nul,
        RuntimeStaticValue::Function(fd) => Value::Fn(Box::new(crate::function::Fn::new(
            Vec::new(),
            vm_instructions_to_optimized(fd.code)
        ))),
        RuntimeStaticValue::Structure(_) => todo!()
    }
}

pub fn vm_instruction_to_optimized(v: VMInstruction) -> Inst {
    return match v {
        VMInstruction::PushLiteral(l) => InstBuilder::new()
            .opcode(Opc::PushLiteral)
            .value(runtime_static_value_to_vm(l))
            .build(),

        VMInstruction::AssignCopy { reg, new } => InstBuilder::new()
            .opcode(Opc::AssignCopy)
            .index(reg)
            .index(new)
            .build(),

        VMInstruction::AssignLiteral { reg, new } => InstBuilder::new()
            .opcode(Opc::AssignLiteral)
            .index(reg)
            .value(runtime_static_value_to_vm(new))
            .build(),

        VMInstruction::AddLeft(l, r) => InstBuilder::new().opcode(Opc::Addl).index(l).index(r).build(),
        VMInstruction::AddRegs(l, r) => InstBuilder::new().opcode(Opc::Add).index(l).index(r).build(),
        VMInstruction::SubLeft(l, r) => InstBuilder::new().opcode(Opc::Subl).index(l).index(r).build(),
        VMInstruction::SubRegs(l, r) => InstBuilder::new().opcode(Opc::Sub).index(l).index(r).build(),
        VMInstruction::MulLeft(l, r) => InstBuilder::new().opcode(Opc::Mull).index(l).index(r).build(),
        VMInstruction::MulRegs(l, r) => InstBuilder::new().opcode(Opc::Mul).index(l).index(r).build(),
        VMInstruction::DivLeft(l, r) => InstBuilder::new().opcode(Opc::Divl).index(l).index(r).build(),
        VMInstruction::DivRegs(l, r) => InstBuilder::new().opcode(Opc::Div).index(l).index(r).build(),
        VMInstruction::ModLeft(l, r) => InstBuilder::new().opcode(Opc::Modl).index(l).index(r).build(),
        VMInstruction::ModRegs(l, r) => InstBuilder::new().opcode(Opc::Mod).index(l).index(r).build(),

        VMInstruction::Break => InstBuilder::new().opcode(Opc::Break).build(),
        VMInstruction::Continue => InstBuilder::new().opcode(Opc::Continue).build(),

        VMInstruction::Call(lhs, args) => {
            let mut i = InstBuilder::new().opcode(Opc::Call).index(lhs);

            for arg in args {
                i = i.index(arg);
            }

            return i.build()
        },

        _ => unimplemented!()
    }
}

pub fn vm_instructions_to_optimized(a: Vec<VMInstruction>) -> Vec<Inst> {
    return a
        .into_iter()
        .map(|f| vm_instruction_to_optimized(f))
        .collect()
}

mod tests;
