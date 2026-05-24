use ospl_common::{ast::{Expression, Type}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, CEData, Compiler, EvalResult, Res, TypeExpectation};

impl Compiler {
    pub fn c_index(
        &mut self,
        l: &Expression,
        r: &Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let left = self.eval(l, ob)?;
        let right = self.eval(r, ob)?;
        if !self.rt(self.stack.top(), &left.ty, l)?.is_indexable() {
            todo!("TODO unwrap - unindexable");
        }

        let return_type = match left.ty {
            Type::List(lty) => *lty,
            Type::Str => Type::Char,
            _ => unreachable!("you didn't handle all the indexable cases!")
        };

        ob.push(InstBuilder::new()
            .opcode(Opc::Index)
            .index(left.address)
            .index(right.address)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: return_type,
        })
    }

    pub fn c_slice(
        &mut self,
        l: &Expression,
        r1: &Expression,
        r2: &Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let left = self.eval(l, ob)?;
        let right_start = self.eval(r1, ob)?;
        let right_end = self.eval(r2, ob)?;
        if !self.rt(self.stack.top(), &left.ty, r1)?.is_sliceable() {
            todo!("TODO unwrap - unsliceable");
        }

        ob.push(InstBuilder::new()
            .opcode(Opc::Slice)
            .index(left.address)
            .index(right_start.address)
            .index(right_end.address)
            .build());

        return Ok(EvalResult {
            address: self.next_var(),
            ty: left.ty,
        })
    }

    pub fn array_pop_top(
        &mut self,
        l: &Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let left = self.array_op_start(l, ob)?;
        ob.push(InstBuilder::new()
            .opcode(Opc::Decrement)
            .index(left.address)
            .build());

        return Ok(left)
    }

    fn array_op_start(
        &mut self,
        l: &Expression,
        ob: &mut Vec<Inst>
    ) -> Res<EvalResult>
    {
        let left = self.eval(l, ob)?;
        if !matches!(left.ty, Type::List(_)) {
            /* error */
            return Err(CE {
                at: Box::new(l.clone()),
                during: "array operation - beginning - type check",
                error: CEData::MismatchedTypes {
                    expected: TypeExpectation::Indexable,
                    got: left.ty
                },
                msg: None
            })
        }

        return Ok(left)
    }
}