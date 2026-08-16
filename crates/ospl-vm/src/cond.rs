//! This module presents implementations for conditionals and control flow on
//! [`VM`], such as if statements, loops, selections, checks, and truthiness.

use ospl_common::inst::{RT, optimized::Inst};

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
        yes: &[Inst],
        no: &[Inst],
    ) -> Control {
        let control = if self.get_truthiness(cond) {
            self.stack.push_parental();
            let out = self.run_all(yes);

            #[cfg(debug_assertions)] let _dbg = crate::debug::DbgMark::new(format!("true if done: {out:?}"));

            // the new if statement scope has a variable we're trying to return
            // that doesn't exist anymore because the scope ended.
            self.stack.end();
            out
        } else {
            self.stack.push_parental();
            let out = self.run_all(no);

            #[cfg(debug_assertions)] let _dbg = crate::debug::DbgMark::new(format!("false if done: {out:?}"));

            self.stack.end();
            out
        };

        return control
    }

    pub fn run_in_parental(
        &mut self,
        code: &[Inst]
    ) -> Control {
        self.stack.push_parental();
        match self.run_all(code) {
            Control::Break => {
                self.stack.end();
                return Control::Default
            }
            Control::Continue => {
                self.stack.end();
                return Control::Continue
            },
            Control::Default => {},
            other => return other
        }
        self.stack.end();

        return Control::Default;
    }

    /// Runs the given code forever until a [`Control::Break`] is issued.
    pub fn run_loop(
        &mut self,
        code: &[Inst]
    ) -> Control {
        loop {
            match self.run_in_parental(code) {
                Control::Continue => {
                    continue;
                },
                _ => {}
            }
        }
    }
}
