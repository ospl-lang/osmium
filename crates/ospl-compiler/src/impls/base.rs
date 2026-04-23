use ospl_common::{
    ast::{class::instance::Entity, repr::{Expr, LValue, StaticValue, Stmt}},
    inst::{RuntimeStaticValue, unoptimized::VMInstruction}
};

use crate::{Compiler, base::{CompilerError, Eval, Res}};

impl Compiler {
    pub fn static_to_runtime(&mut self, x: &StaticValue) -> Res<RuntimeStaticValue> {
        return match x {
            StaticValue::Int(i) => Ok(RuntimeStaticValue::Int(*i)),
            StaticValue::Float(f) => Ok(RuntimeStaticValue::Float(*f)),
            StaticValue::Nul => Ok(RuntimeStaticValue::Nul),

            StaticValue::Function(fd) => Ok(
                RuntimeStaticValue::Function(
                    self.compile_fn(fd)?
                )
            ),

            _ => unimplemented!(),
        }
    }

    pub fn eval(
        &mut self,
        expr: &Expr,
        out: &mut Vec<VMInstruction>
    ) -> Res<Eval> {
        match expr {
            Expr::StaticLiteral(x) => self.static_literal_real(x, out),
            Expr::BinaryOp { op, lhs, rhs } => self.binaryop(op, lhs, rhs, out),
            Expr::Call(func, args) => self.call_fn(func, args, out),
            Expr::LValue(lv) => {
                let lv = self.get_lvalue(lv)?;
                return Ok(Eval {
                    index: lv.get_index(),
                    ty: lv.get_type().clone(),   // FIX-CLONE: undesirable...
                })
            },
            Expr::Construct(vp) => self.vp_instantiate_class(vp, out),
        }
    }

    pub fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        out: &mut Vec<VMInstruction>
    ) -> Res<()> {
        return match stmt {
            Stmt::DeclareReal { lhs, rhs } => match rhs {
                Some(x) => self.declare_real(lhs, x, out),
                None => unimplemented!()
            },
            Stmt::AssignReal { lhs, rhs } => self.assign_real(lhs, rhs, out),
            Stmt::Return(e) => self.ret(e, out),
            _ => unimplemented!()
        };
    }

    pub fn static_literal_real(
        &mut self,
        x: &StaticValue,
        out: &mut Vec<VMInstruction>
    ) -> Res<Eval> {
        let compiled = self.static_to_runtime(x)?;
        out.push(VMInstruction::PushLiteral(compiled));

        return Ok(Eval {
            index: self.top_mut()?.next_pre(),
            ty: x.clone().into()
        })
    }

    pub fn compile_stmt_list(
        &mut self,
        stmts: &[Stmt],
        out: &mut Vec<VMInstruction>
    ) -> Res<()> {
        for stmt in stmts {
            self.compile_stmt(&stmt, out)?;
        }

        return Ok(())
    }

    pub fn get_lvalue_index(&self, lv: &LValue) -> Res<usize> {
        return Ok(match lv {
            LValue::Var(v) => self.top()?.get_local(v)?.get_index(),
            _ => todo!(),
        })
    }

    pub fn get_lvalue(&self, lv: &LValue) -> Res<&dyn Entity> {
        return Ok(match lv {
            LValue::Var(v) => self.top()?.get_local(v)?,
            LValue::Property(lv, n) => {
                let rlv = self.get_lvalue(lv)?;
                let Some(x) = rlv.get_type().get_property(&*n)
                    else { return Err(CompilerError::NotFoundInScope { id: n.to_string() }) };

                return Ok(x)
            }
        })
    }
}
