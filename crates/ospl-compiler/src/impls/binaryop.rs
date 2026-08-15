use ospl_common::{ast::{Type, ops::{AssignOp, BinaryOp, BinaryOpType}}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res};

impl Compiler {
    pub fn binary_op(
        &mut self,
        b: &BinaryOp,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let left = self.eval(&b.left, ob)?;
        let right = self.eval(&b.right, ob)?;

        // don't do the type checking for the question mark operator
        match (&left.ty, &right.ty, &b.kind) {
            // numeric ops: only same-type allowed
            (Type::Int, Type::Int, op) if op.is_integral() => {}
            (Type::Address, Type::Address, op) if op.is_integral() => {}
            (Type::Float, Type::Float, op) if op.is_floating_point() => {}

            (Type::List(_), _, BinaryOpType::Gt) => {},
            (Type::List(_), _, BinaryOpType::Lt) => {},
            (Type::List(_), _, BinaryOpType::LShift) => {},
            (Type::List(_), _, BinaryOpType::RShift) => {},
            (Type::List(_), _, BinaryOpType::Subtract) => {},
            (Type::List(_), _, BinaryOpType::Equals) => {},
            (Type::List(_), _, BinaryOpType::NotEquals) => {},

            // simple exact matches for non-numeric ops
            (Type::Bool, Type::Bool, BinaryOpType::Equals) => {}
            (Type::Bool, Type::Bool, BinaryOpType::NotEquals) => {}
            (Type::Bool, Type::Bool, BinaryOpType::And) => {},
            (Type::Bool, Type::Bool, BinaryOpType::Or) => {},
            (Type::Bool, Type::Bool, BinaryOpType::Xor) => {},

            (Type::Str, Type::Str, BinaryOpType::Add) => {}
            (Type::Str, Type::Str, BinaryOpType::Equals) => {}
            (Type::Str, Type::Str, BinaryOpType::NotEquals) => {}

            // fallback mismatch
            (lty, rty, _) if lty != rty => {
                return Err(CE {
                    at: Box::new(b.left.clone()),
                    msg: Some("Refer to The Absolute Guide to OSPL, or ospl-compiler/src/impls/binaryop.rs, for valid operations"),
                    during: "binary operation - type check",
                    error: CEData::MismatchedTypes {
                        expected: crate::TypeExpectation::Exact(left.ty.clone()),
                        got: right.ty.clone(),
                    }
                });
            }

            _ => {}
        }

        let opc = match b.kind {
            BinaryOpType::Add       => Opc::Add,
            BinaryOpType::Subtract  => Opc::Sub,
            BinaryOpType::Multiply  => Opc::Mul,
            BinaryOpType::Divide    => Opc::Div,
            BinaryOpType::Modulo    => Opc::Mod,
            BinaryOpType::Equals    => Opc::Eq,
            BinaryOpType::NotEquals => Opc::Neq,
            BinaryOpType::Gt        => Opc::Gt,
            BinaryOpType::Lt        => Opc::Lt,
            BinaryOpType::Ge        => Opc::Gte,
            BinaryOpType::Le        => Opc::Lte,
            BinaryOpType::And       => Opc::And,
            BinaryOpType::Or        => Opc::Or,
            BinaryOpType::Xor       => Opc::Xor,
            BinaryOpType::LShift    => Opc::LShift,
            BinaryOpType::RShift    => Opc::RShift,
        };

        let inst = InstBuilder::new()
            .opcode(opc)
            .index(left.address)
            .index(right.address)
            .build();

        ob.push(inst);

        return Ok(EvalResult {
            address: self.next_var(),

            // we use right here because `?` returns its right operand (or
            // undefined) as the type, and all others binary ops have the
            // same left and right return type, so we'll use the right.
            ty: right.ty.clone()
        })
    }

    pub fn assign_op(
        &mut self,
        b: &AssignOp,
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        let left = self.get_lvalue(&b.left, ob)?;
        let right = self.eval(&b.right, ob)?;

        let opc = match &b.kind {
            BinaryOpType::Add       => Opc::Addl,
            BinaryOpType::Subtract  => Opc::Subl,
            BinaryOpType::Multiply  => Opc::Mull,
            BinaryOpType::Divide    => Opc::Divl,
            BinaryOpType::Modulo    => Opc::Modl,
            other => return Err(CE {
                at: Box::new(b.left.clone()),
                error: CEData::InvalidAssignOp { op: other.clone() },
                during: "assign operation (illegal)",
                msg: Some("perhaps you want to use def `X = X op Y`")
            })
        };

        let inst = InstBuilder::new()
            .opcode(opc)
            .index(left.address)
            .index(right.address)
            .build();

        ob.push(inst);

        return Ok(())
    }
}
