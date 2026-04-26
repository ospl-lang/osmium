use ospl_common::{ast::{Statement, repr::Stmt}, inst::optimized::Inst};

pub struct Compiler;

impl Compiler {
    pub fn process_stmt(
        &mut self,
        s: &Statement,
        ob: &mut Vec<Inst>,
    ) {
        match s.inner {
            Stmt::ReturnScope => {

            },
            _ => unimplemented!()
        }
    }
}
