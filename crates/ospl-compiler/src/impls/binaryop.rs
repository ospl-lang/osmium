use ospl_common::{ast::{Type, ops::{AssignOp, BinaryOp, BinaryOpType}}, inst::optimized::{InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res};

impl<'a> Compiler<'a> {
    pub fn binary_op(
        &mut self,
        b: &BinaryOp,
    ) -> Res<EvalResult>
    {
        let left = self.eval(&b.left)?;
        let right = self.eval(&b.right)?;

        // don't do the type checking for the question mark operator
        match (&left.ty, &right.ty, &b.kind) {
            // question operator bypasses type equality rules
            (_, _, BinaryOpType::Question) => {}
            (Type::Function(_), Type::Function(f2), BinaryOpType::Add) => {
                if f2.args.len() != 0 || f2.ret != Type::Nul {
                    todo!("ERROR")
                }
            }

            // numeric ops: only same-type allowed
            (Type::Int, Type::Int, op) if op.supports_numerical() => {}
            (Type::Address, Type::Address, op) if op.supports_numerical() => {}
            (Type::Float, Type::Float, op) if op.supports_numerical() => {}

            // simple exact matches for non-numeric ops
            (Type::Bool, Type::Bool, BinaryOpType::Equals) => {}
            (Type::Bool, Type::Bool, BinaryOpType::NotEquals) => {}

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
            BinaryOpType::Question  => Opc::QuestionMark,
        };

        let inst = InstBuilder::new()
            .opcode(opc)
            .index(left.address)
            .index(right.address)
            .build();

        self.insts.push(inst);

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
    ) -> Res<()>
    {
        let left = self.get_lvalue(&b.left)?;
        let right = self.eval(&b.right)?;

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

        self.insts.push(inst);

        return Ok(())
    }
}
