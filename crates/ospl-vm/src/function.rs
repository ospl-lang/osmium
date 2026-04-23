use crate::{Control, inst::optimized::Inst};

use super::{Frame, VM, arena::ArenaIndex};

#[derive(Debug, Clone)]
pub struct Fn {
    /// The indexes into the arena that this thing can thingy agghgh idk
    pub lexical_indexes: Vec<ArenaIndex>,

    /// The code of this function
    pub code: Vec<Inst>,
}

impl Fn {
    /// Creates a new function given lexical indexes and code.
    pub fn new(lexical_indexes: Vec<ArenaIndex>, code: Vec<Inst>) -> Self {
        return Fn {
            lexical_indexes,
            code,
        }
    }

    /// Creates a new instance that can be used for literals. Whatever that may mean
    pub fn new_literal(lexical_indexes: Vec<ArenaIndex>, code: Vec<Inst>) -> Box<Self> {
        return Box::new(Self::new(lexical_indexes, code))
    }
}

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
    /// frame SP $00 --> | arg 1: copy of original value
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
        // the clones here are tiny and insignificant. C programmers don't need
        // them, because they'd never write this function in this way, because
        // they're insane people. But we need them because we write this
        // function in this way because we aren't masochists who subject
        // ourselves to manually writing every single collection type, and then
        // have to wonder why it's leaking memory all the sudden.
        // 
        // Besides my rant on C programmers, I know the alternative to this
        // (not cloning and using the reference) is safe, rustc, however,
        // doesn't know that. I'm unable to tell it that, because unsafe{}
        // doesn't actually let you bypass reference scemantics. They should
        // add some form of that though (maybe not through unsafe blocks...)
        // maybe via `ref unsafe` blocks: the `ref` keyword already exists, and
        // is underused, why not use it?
        // 
        // We pay the price of tiny memcpy()s for safety, praise Ferris the
        // crab!
        //
        // -Colton

        // ignore the above rant because I wrote unsafe code to fix the
        // dumb clones!
        // -Colton, a few weeks later

        // here, it is important that we push the lexical regs to the frame
        // AFTER we push the arguments, this is just the calling convention
        // we're gonna use, because it makes things easier for you and the
        // compiler.

        let mut frame = Frame::default();

        // ARGUMENTS
        {
            let top = self.top();
            for arg in args {
                let abs = &top.indexes[*arg];
                frame.indexes.push(*abs);
            }
        };

        // LEXICALS
        let Some(f) = self.get_value_top(f).as_fn()
            else { panic!("cannot call this object") };

        frame.indexes.extend_from_slice(f.lexical_indexes.as_slice());

        // now, since Rust sucks, we're gonna do unsafe
        // SAFETY: I PROMISE THAT `f.code` AND IT'S PARENTS WILL NOT BE MUTATED
        unsafe {
            let very_good_safe = &raw const f.code;
            self.push_frame(frame);  // needs to be in unsafe because of course it does..

            for inst in &*very_good_safe {
                let run = self.run_one_optimized(inst);
                match run {
                    Control::Return(i) => self.ret(i),
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
}
