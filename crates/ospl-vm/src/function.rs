use ospl_common::{ast::frame::RuntimeFrame, types::AbsAddress};
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
        let f = self.get_value_top(f);
        let f = match f.as_fn() {
            Some(o) => o,
            None => panic!("can't call object of type {f:?}")
        };

        frame.indexes.extend_from_slice(f.lexical_indexes.as_slice());

        // INVARIANT: I guarantee that the number of args passed in matches the
        // function's expectations. If this invariant is broken, then the OSPL
        // function (and any function using the returned scope of the function)
        // may experience undefined behaviour.
        // now, since Rust sucks, we're gonna do unsafe
        // SAFETY: I PROMISE THAT `f.code` AND ITS PARENTS WILL NOT BE MUTATED
        unsafe {
            let very_good_safe = &raw const f.code;
            self.push_frame(frame);  // needs to be in unsafe because of course it does..

            for inst in &*very_good_safe {
                let run = self.run_one(inst);
                match run {
                    Control::Return(i) => {
                        self.ret(i);
                        break;
                    },
                    Control::ReturnScope => {
                        self.retscope();
                        break;
                    },
                    _ => {},
                }
            }
        }

        return Control::Default
    }

    /// Returns a copy of the value to the previous stack frame
    pub fn ret(&mut self, address: AbsAddress) {
        unsafe {
            let f = self.pop_scope_without_gc();

            // don't touch anything here (this is stupid but trust me
            // it will work)
            self.arena.inc_refcount(address);
            self.arena.gc_frame_destroyed(&f.indexes);
        }

        self.top_mut().indexes.push(address);
    }

    /// Returns the current frame as a value
    pub fn retscope(&mut self) {
        // may or may not work...
        let s = unsafe{self.pop_scope_without_gc()};
        self.push_literal(RuntimeValue::Scope(s));
    }

    pub fn call_foreign_function(&mut self, h: usize, idxs: &[usize]) {
        unsafe {
            let RuntimeValue::ForeignFn(h) = *self.get_value_top(h)
                else { unimplemented!("not a foreign fn") };

            let x = &raw const *self.ffi.get_function(h).expect("FFI function not found");

            let mut values = Vec::new();
            for idx in idxs {
                values.push(self.get_value_top(*idx).clone());
            }

            let ret = match crate::ffi::call_foreign_function(
                &*x,
                &values,
            ) {
                Ok(ret) => ret,
                Err(e) => panic!("failed to call FFI function {:?}\n{:?}", *x, e),
            };

            self.push_literal(ret);
        }
    }
}
