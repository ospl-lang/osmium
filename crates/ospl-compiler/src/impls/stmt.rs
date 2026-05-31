use ospl_common::{ast::Type, inst::optimized::{Inst, InstBuilder, Opc}};

pub enum Control {
    Default,
    Break,
    Continue,
    Return(usize),
    ReturnScope,
}

use crate::{CE, CEData, Compiler, Res, ast::{Statement, Stmt}};

impl Compiler {
    pub fn compile_stmt(
        &mut self,
        s: &Statement,
        ob: &mut Vec<Inst>
    ) -> Res<Control>
    {
        // tracing::trace!(stmt=?s, "compiling stmt");
        // note: we never match on Control so it doesn't really matter
        match &*s.inner {
            // TODO: check type
            Stmt::Define(dcl) => {
                let _span = tracing::debug_span!("dcl", id=dcl.name);
                let _enter = _span.enter();
                let eval = self.eval(&dcl.rhs, ob)?;

                self.stack.top_mut().declare(dcl.name.to_string(), eval.address, eval.ty);
            },

            Stmt::DefineTypeAlias(dcl) => {
                let t = self.rt(self.stack.top(), &dcl.ty, s)?;
                self.stack.top_mut().declare_non_addressable(dcl.name.clone(), t);
            },

            Stmt::DefineNominalTypeAlias(dcl) => {
                let t = self.rt(self.stack.top(), &dcl.ty, s)?;
                self.stack.top_mut().declare_non_addressable(dcl.name.clone(), t);
            }

            Stmt::Expr(e) => {
                let _ = self.eval(e, ob)?;
            },

            Stmt::ReturnScope => {
                let inst = InstBuilder::new()
                    .opcode(Opc::RetScope)
                    .build();
                
                ob.push(inst);
                return Ok(Control::ReturnScope)
            },

            Stmt::Return(e) => {
                let eval = self.eval(e, ob)?;
                ob.push(InstBuilder::new().opcode(Opc::Ret).index(eval.address).build());
                return Ok(Control::Return(eval.address))
            },

            Stmt::If(left, yes, no) => 
                self.compile_if_stmt(left, yes, no, ob)?,

            Stmt::Break => {
                ob.push(InstBuilder::new().opcode(Opc::Break).build());
                return Ok(Control::Break)
            },

            Stmt::Continue => {
                ob.push(InstBuilder::new().opcode(Opc::Continue).build());
                return Ok(Control::Continue)
            },

            Stmt::Loop(l) => self.compile_loop_stmt(l, ob)?,

            Stmt::Assign(lv, to) => {
                // BUGNOTE: I'm unsure if this get_lvalue() call is safe
                // because we're going to interpret it under the current
                // scope. get_lvalue may or may not work this way and I
                // honestly forgot.
                let reval = self.eval(to, ob)?;
                let leval = self.get_lvalue(lv, ob)?;
                // don't check if we're currently of undefined type
                if leval.ty != Type::Undefined {
                    if leval.ty != reval.ty {
                        return Err(CE {
                            at: Box::new(lv.clone()),
                            msg: Some("perhaps wrap the right-hand side's type?"),
                            during: "assignment operation",
                            error: CEData::MismatchedTypes { expected: crate::TypeExpectation::Exact(leval.ty), got: reval.ty }
                        })
                    }
                }

                let i = InstBuilder::new()
                    .opcode(Opc::AssignRef)
                    .index(leval.address)
                    .index(reval.address)
                    .build();

                ob.push(i);
            },
            Stmt::AssignOp(b) => {
                self.assign_op(b, ob)?;
            },
        }

        return Ok(Control::Default)
    }

    pub fn compile_block(
        &mut self,
        s: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        for stmt in s {
            self.compile_stmt(stmt, ob)?;
        }

        return Ok(())
    }

    /// [`Self::compile_block`] with better guarantees:
    /// - it guarantees a new scope is not created
    /// - it guarantees the current scope is not modified
    /// - it guarantees it behaves like a for loop emiting
    ///     instructions in a list would
    pub fn compile_all(
        &mut self,
        s: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        return self.compile_block(s, ob);
    }
}