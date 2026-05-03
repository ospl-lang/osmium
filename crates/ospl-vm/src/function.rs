use ospl_common::ast::frame::RuntimeFrame;
use crate::{Control, RuntimeValue};
use super::{VM, arena::ArenaIndex};

impl VM {
    /// Calls a function with the OSPL calling convention.
    /// 
    /// # Parameters
    /// - `f`: index to the function, in the current scope
    /// - `args`: list of indexes to the arguments of the function, in the
    ///           current scope
    /// 
    /// # Calling convention
    /// the calling convention for functions is as follows
    /// ```text,no_run
    /// frame SP $00 --> | arg 1: ref to original value
    ///          $01     | arg 2
    ///          $02     | arg 3
    ///          $03     | arg 4
    ///          $04     | arg 5
    ///                  |
    ///                  | ... more arguments follow ...
    ///                  | 
    ///          $05     | lexically scoped variable A
    ///          $06     | lexically scoped variable B
    ///          $07     | lexically scoped variable C
    ///                  | 
    ///                  | ... more captures follow ...
    ///                  | 
    ///          $08     | local variable A
    ///          $09     | local variable B
    ///          $10     | local variable C
    ///                  | 
    ///                  | ... more locals follow ...
    ///                  | 
    /// ```
    /// 
    /// OSPL passes all arguments by reference, and all returns by value.
    /// 
    /// # Safety
    /// YOU CANNOT MUTATE THE CODE OF THE FUNCTION YOU ARE CALLING, OR ANY PARENT FUNCTION,
    /// WITHOUT UNDEFINED BEHAVOIUR. PLEASE DO NOT DO THIS! IT'S A VERY BAD IDEA!!
    pub fn call_fn(&mut self, f: usize, args: &[ArenaIndex]) -> Control {
        // ignore the above rant because I wrote unsafe code to fix the
        // dumb clones!

        // here, it is important that we push the lexical regs to the frame
        // AFTER we push the arguments, this is just the calling convention
        // we're gonna use, because it makes things easier for you and the
        // compiler.

        let mut frame = RuntimeFrame::default();

        // ARGUMENTS
        {
            let top = self.top();
            for arg in args {
                let abs = top.indexes[*arg];
                frame.indexes.push(abs);
            }
        };

        // LEXICALS
        let Some(f) = self.get_value_top(f).as_fn()
            else { panic!("cannot call this object") };

        frame.indexes.extend_from_slice(f.lexical_indexes.as_slice());

        // INVARIANT: I guarantee that the number of args passed in matches the
        // function's expectations. If this invariant is broken, then the OSPL
        // function (and any function using the returned scope of the function)
        // may experience undefined behaviour.
        frame.num_args = args.len();

        frame.num_captures = f.lexical_indexes.len();

        // now, since Rust sucks, we're gonna do unsafe
        // SAFETY: I PROMISE THAT `f.code` AND ITS PARENTS WILL NOT BE MUTATED
        unsafe {
            let very_good_safe = &raw const f.code;
            self.push_frame(frame);  // needs to be in unsafe because of course it does..

            for inst in &*very_good_safe {
                let run = self.run_one(inst);
                match run {
                    Control::Return(i) => self.ret(i),
                    Control::ReturnScope => self.retscope(),
                    _ => {},
                }
            }
        }

        return Control::Default
    }

    /// Returns a copy of the value to the previous stack frame
    pub fn ret(&mut self, i: ArenaIndex) {
        let vr = self.arena.get(self.top().indexes[i]).clone();
        self.end_scope();

        self.push_literal(vr);
    }

    /// Returns the current frame as a value
    pub fn retscope(&mut self) {
        // may or may not work...
        let s = self.pop_scope();
        self.push_literal(RuntimeValue::Scope(Box::new(s)));
    }
}
