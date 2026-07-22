use ospl_common::{ast::frame::RuntimeFrame, inst::{assume, make}, types::AbsAddress};
use crate::Control;
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

        let (c_start, c_end) = {
            let f = self.get_value_top(at);
            let f = match f.as_fn() {
                Some(o) => o,
                None => {
                    panic!("VM error: can't call object of type {f:?} | at={at} | top={:?}", self.stack.top_indexes());
                },
            };

            // CAPTURES
            frame.indexes.extend_from_slice(f.captures.as_slice());
            f.code
        };

        // ARGUMENTS
        {
            let top = self.stack.top_indexes();
            for arg in args {
                let Some(abs) = top.get(*arg)
                else {
                    // this is a bug don't remove the crash
                    panic!("VM bug: argument {arg:?} was out of bounds! len={} | frame={frame:?}", top.len());
                };
                frame.indexes.push(*abs);
            }
        };

        self.stack.push_frame_value(frame, self.instruction_pointer);

        unsafe {
            let x = &raw const self.instruction_stream[c_start..c_end];

            for inst in &*x {
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
    /// and switches the instruction pointer accordingly
    pub fn ret(&mut self, address: AbsAddress) {
        let _ = self.stack.end();
        self.stack.top_add_index(address);
    }

    /// Returns the current frame as a value
    pub fn retscope(&mut self) {
        // may or may not work...
        let s = self.stack.pop();

        self.push_literal(make::scope(s));
    }

    pub fn call_foreign_function(&mut self, h: usize, idxs: &[usize]) {
        unsafe {
            let Some(h) = assume::foreignfun(self.get_value_top(h))
                else { unimplemented!("not a foreign fn") };

            let x = &raw const *self.ffi.get_function(*h).expect("FFI function not found");

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
