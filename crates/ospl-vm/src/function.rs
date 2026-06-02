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
    /// frame SP $00 --> | capture 1: ref to original value
    ///          $01     | capture 2
    ///          $02     | capture 3
    ///                  |
    ///                  | ... more captures follow ...
    ///                  | 
    ///          $03     | argument 1: ref to original value
    ///          $04     | argument 2
    ///          $05     | argument 3
    ///                  | 
    ///                  | ... more arguments follow ...
    ///                  | 
    ///          $06     | local variable A
    ///          $07     | local variable B
    ///          $18     | local variable C
    ///                  | 
    ///                  | ... more locals follow ...
    ///                  | 
    /// ```
    /// 
    /// OSPL passes all arguments by reference, and all returns by reference as well.
    pub fn call_fn(&mut self, at: usize, args: &[ArenaIndex]) -> Control {
        let mut frame = RuntimeFrame::default();

        let f = self.get_value_top(at);
        let f = match f.as_fn() {
            Some(o) => o,
            None => panic!("can't call object of type {f:?} | at={at} | top={:?}", self.top())
        };

        // CAPTURES
        frame.indexes.extend_from_slice(f.captures.as_slice());

        // ARGUMENTS
        {
            let top = self.top();
            for arg in args {
                let abs = top.indexes.get(*arg).unwrap_or_else(|| {
                    panic!("argument {arg:?} was out of bounds! len={} | frame={frame:?}", top.indexes.len());
                });
                frame.indexes.push(*abs);
            }
        };

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

    /// Returns the value to the previous stack frame
    pub fn ret(&mut self, address: AbsAddress) {
        let _ = self.pop_scope();
        self.top_mut().indexes.push(address);
    }

    /// Returns the current frame as a value
    pub fn retscope(&mut self) {
        // may or may not work...
        let s = self.pop_scope();

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
