//! This module presents implementations for conditionals and control flow on
//! [`VM`], such as if statements, loops, selections, checks, and truthiness.

use ospl_common::inst::optimized::Inst;

use crate::{Control, VM, RuntimeValue, arena::ArenaIndex};

impl VM {
    /// Returns the truthiness of a given value.
    #[inline(always)]
    pub fn get_truthiness(
        &self,
        indx: ArenaIndex
    ) -> bool {
        let value = self.get_value_top(indx);

        // this code is slightly more CPU friendly than a match.
        // bool at the top, because it's the most common truth type.
        if let RuntimeValue::Bool(b) = value { return *b }

        if matches!(value, RuntimeValue::Int(0) | RuntimeValue::Float(0.0) | RuntimeValue::Nul | RuntimeValue::Undefined) {
            return false
        }

        return true

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
            self.new_scope_parental();
            let out = self.run_all(yes);

            // the new if statement scope has a variable we're trying to return that doesn't exist anymore because the scope ended.
            self.end_scope();
            out
        } else {
            self.new_scope_parental();
            let out = self.run_all(no);
            self.end_scope();
            out
        };

        return control
    }

    /// Runs the given code forever until a [`Control::Break`] is issued.
    pub fn run_loop(
        &mut self,
        code: &[Inst]
    ) -> Control {
        loop {
            self.new_scope_parental();
            match self.run_all(code) {
                Control::Break => break,
                Control::Default => {},
                other => {
                    self.end_scope();
                    return other
                },
            }
            self.end_scope();
        }

        self.end_scope();
        return Control::Break
    }
}