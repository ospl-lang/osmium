//! This module presents implementations for conditionals and control flow on
//! [`VM`], such as if statements, loops, selections, checks, and truthiness.

use ospl_common::inst::RT;

use crate::{Control, VM, arena::ArenaIndex};

impl VM {
    /// Returns the truthiness of a given value.
    #[inline(always)]
    pub fn get_truthiness(
        &self,
        indx: ArenaIndex
    ) -> bool {
        let value = self.get_value_top(indx);

        match value.tag {
            RT::Bool => return unsafe { value.data.bool },
            RT::Int => return unsafe { value.data.int != 0 },
            RT::Addr => return unsafe { value.data.address != 0 },
            RT::Float => return unsafe { value.data.float != 0.0 },
            RT::Undefined => return false,
            RT::Nul => return false,
            _ => return true
        }
    }

    /// Takes a branch, the `yes` branch is taken if `cond` is truthy,
    /// and `no` is taken if it is falsy. Returns the [`Control`] of
    /// the block that is executed.
    #[inline(always)]
    pub fn if_statement(
        &mut self,
        cond: ArenaIndex,
        yes: (usize, usize),
        no: (usize, usize),
    ) -> Control {
        let control = if self.get_truthiness(cond) {
            self.stack.push_parental(self.instruction_pointer);
            self.instruction_pointer = yes.0;
            let out = self.run_forever();

            #[cfg(debug_assertions)] let _dbg = crate::debug::DbgMark::new(format!("true if done: {out:?}"));

            // the new if statement scope has a variable we're trying to return
            // that doesn't exist anymore because the scope ended.
            self.stack.end();
            out
        } else {
            self.stack.push_parental(self.instruction_pointer);
            self.instruction_pointer = no.0;
            let out = self.run_forever();

            #[cfg(debug_assertions)] let _dbg = crate::debug::DbgMark::new(format!("false if done: {out:?}"));

            self.stack.end();
            out
        };

        return control
    }

    /// Runs the given code forever until a [`Control::Break`] is issued.
    pub fn run_loop(
        &mut self,
        code: (usize, usize)
    ) -> Control {
        loop {
            self.stack.push_parental(self.instruction_pointer);
            self.instruction_pointer = code.0;
            match self.run_all(code.1) {
                Control::Break => {
                    self.instruction_pointer = self.stack.end();
                    return Control::Default
                }
                Control::Continue => {
                    self.instruction_pointer = self.stack.end();
                    continue;
                },
                Control::Default => {},
                other => return other
            }
            self.instruction_pointer = self.stack.end();
        }
    }
}
