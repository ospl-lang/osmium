use ospl_common::{ast::{module::VPath, repr::{Expr, LValue, Type}}, inst::unoptimized::VMInstruction};

use crate::{Compiler, base::{CompilerError, Res}};

impl Compiler {
    // Managing "real" types (things that actually get compiled into
    // runtime values)
    pub fn declare_real(
        &mut self,
        lv: &LValue,
        rhs: &Expr,
        out: &mut Vec<VMInstruction>
    ) -> Res<()> {
        let right = self.eval(&rhs, out)?;

        match lv {
            LValue::Var(v) => self.top_mut()?.declare(
                v.clone(),
                right.index,
                right.ty
            ),
            LValue::Property(_, n) => return Err(CompilerError::DeclaringPropertiesIsIllegal {
                property: n.clone()
            })
        }

        return Ok(())
    }

    pub fn assign_real(
        &mut self,
        lv: &LValue,
        rhs: &Expr,
        out: &mut Vec<VMInstruction>
    ) -> Res<()> {
        // TODO: optimize into an AssignLiteral
        let right = self.eval(&rhs, out)?;

        /* I'm unsure if we need to change `entity.reg` here. It seems to be ok
         * without the change, and generally, mutating things after they've
         * been declared is against... really the whole codebase right now (and
         * for good reason!)
         * 
         *  -- PersonP
        */
        let entity = self.get_lvalue(lv)?;
        if right.ty != *entity.get_type() {
            return Err(CompilerError::MismatchedType {
                expected: right.ty,
                got: entity.get_type().clone()
            })
        }

        out.push(VMInstruction::AssignCopy {
            reg: entity.get_index(),
            new: right.index
        });

        return Ok(())
    }

    /// Declares a "fake" type, in a VScope
    pub fn declare_fake(
        &mut self,
        vp: &VPath,
        ty: &Type
    ) -> Res<()> {
        self.vroot.create_member(vp, ty.clone())?;
        return Ok(())
    }
}
