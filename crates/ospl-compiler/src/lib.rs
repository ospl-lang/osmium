use ospl_common::{ast::{Statement, frame::Scope, repr::Stmt}, inst::optimized::Inst};

pub struct Compiler;

impl Compiler {
    pub fn process_stmt(
        &mut self,
        s: &Statement,
        ob: &mut Vec<Inst>,
    ) {
        match s.inner {
            Stmt::ReturnScope => {
                let x = Scope::default();
            },
            _ => unimplemented!()
        }
    }
}
