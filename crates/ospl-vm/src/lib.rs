use arena::{ArenaIndex, Arena};
use ospl_common::{ast::frame::RuntimeFrame, inst::{RuntimeFunction, RuntimeValue, optimized::{Inst, Opc}}};
use crate::{arena::ArenaItem, gc::GcEvent};

mod ffi;
mod cond;
mod binaryops;
mod pushes;
mod function;
pub mod arena;
pub mod gc;
pub mod list;

pub mod tests;


// WE WILL EVENTUALLY SWITCH TO FIXEDVEC. WHEN FIXEDVEC BECOMES
// FAST ENOUGH.
// pub const VM_FRAME_SIZE: usize = 256;

#[derive(Debug)]
pub struct VM {
    pub arena: Arena,
    pub stack: Vec<RuntimeFrame>,
}

#[derive(Debug)]
pub enum Control {
    Break,
    Continue,
    Return(ArenaIndex),
    ReturnScope,
    Default,
}

impl VM {
    pub fn new() -> Self {
        return Self {
            arena: Arena::new(),
            stack: vec![RuntimeFrame::default()],
        }
    }

    pub fn push_frame(&mut self, f: RuntimeFrame) {
        self.arena.gc_event(GcEvent::FrameAdded {
            indexes: &f.indexes
        });
        self.stack.push(f);
    }

    pub fn new_child_of_scope(&mut self, f: &RuntimeFrame) {
        let parents = f.indexes.clone();
        self.arena.gc_event(GcEvent::FrameAdded {
            indexes: parents.as_slice()
        });

        self.stack.push(RuntimeFrame {
            indexes: parents
        });
    }

    #[inline(always)]
    pub fn get_item_top(&mut self, abs: ArenaIndex) -> &mut ArenaItem {
        return self.arena.get_item_mut(self.top().indexes[abs]);
    }

    pub fn new_scope_but_parental(&mut self) {
        let parents = self.top().indexes.clone();
        self.arena.gc_event(GcEvent::FrameAdded {
            indexes: parents.as_slice()
        });

        self.stack.push(RuntimeFrame {
            indexes: parents
        });
    }

    pub fn end_scope(&mut self) {
        let Some(f) = self.stack.pop()
            else { panic!("You can't return from the top level of a script, you fucking moron!") };

        self.arena.gc_event(GcEvent::FrameDestroyed {
            indexes: f.indexes.as_slice()
        });
    }

    /// Returns the scope without decrementing the refcount
    #[inline(always)]
    pub fn pop_scope(&mut self) -> RuntimeFrame {
        return self.stack.pop()
            .unwrap_or_else(|| panic!("You can't return from the top level of a script, you fucking moron!"));
    }

    #[inline(always)]
    pub fn top_mut(&mut self) -> &mut RuntimeFrame {
        return self.stack.last_mut().unwrap()
    }

    #[inline(always)]
    pub fn top(&self) -> &RuntimeFrame {
        return self.stack.last().unwrap()
    }

    /// Pushes the value onto the next register in the stack
    /// 
    /// Is about equivalent to literal assignemnt.
    #[inline(always)]
    pub fn push_literal(&mut self, v: RuntimeValue) -> ArenaIndex {
        let i = self.arena.push(v);
        self.top_mut().indexes.push(i);
        return i
    }

    /// Returns a reference to a value given an index, using the top frame's
    /// index array
    fn get_value_top(&self, i: ArenaIndex) -> &RuntimeValue {
        return self.arena.get(self.top().indexes[i])
    }

    unsafe fn raw_get_value_top(&self, i: ArenaIndex) -> *const RuntimeValue {
        return unsafe{ self.arena.raw_get(self.top().indexes[i]) }
    }

    unsafe fn raw_get_value_top_mut(&mut self, i: ArenaIndex) -> *mut RuntimeValue {
        return unsafe{ self.arena.raw_get_mut(self.top().indexes[i]) }
    }

    /// Returns a reference to a value given an index, using the top frame's
    /// index array
    #[allow(dead_code)]
    fn get_mut_value_top(&mut self, i: ArenaIndex) -> &mut RuntimeValue {
        return self.arena.get_mut(self.top().indexes[i])
    }

    fn assign_copy(&mut self, reg: usize, new: usize) {
        let x = self.get_value_top(new).clone();
        let b = self.get_mut_value_top(reg);
        *b = x;
    }

    pub fn run_one(&mut self, inst: &Inst) -> Control {
        // you're about to see a lot of unsafe code!
        //
        // It's done to improve preformance. Rust adds
        // a bunch of bounds-checking to Vec<T> indexes,
        // but we know that it's safe as long as our
        // instructions are valid. Rust doesn't know
        // this, so we have to tell it ourselves, which
        // requires this unsafe code.

        unsafe { match &inst.opcode {
            Opc::PushLiteral => {
                self.push_copy(
                    inst.immediate
                        .as_ref()
                        .unwrap()
                );
            },

            Opc::PushFunction => {
                let new_indexes = inst.indexes.iter().map(|x| {
                    // RelAddr -> AbsAddr
                    return self.top().indexes[*x]
                }).collect();
                let f_code = inst.children.get_unchecked(0);
                
                let f = RuntimeFunction {
                    lexical_indexes: new_indexes,
                    code: f_code.clone()
                };

                self.push_literal(RuntimeValue::Function(f));
                return Control::Default
            }

            Opc::If => return self.if_statement(
                *inst.indexes.get_unchecked(0),
                &inst.children.get_unchecked(0),
                &inst.children.get_unchecked(1),
            ),

            Opc::AssignCopy => self.assign_copy(*inst.indexes.get_unchecked(0), *inst.indexes.get_unchecked(1)),

            Opc::AssignLiteral => {
                // don't even fuck with this one lmao.
                *self.arena.get_mut(self.top().indexes[inst.indexes[0]]) = inst.immediate.as_ref().unwrap().clone();
            },

            Opc::Add => self.add_regs(*inst.indexes.get_unchecked(0), *inst.indexes.get_unchecked(1)),
            Opc::Sub => self.sub_regs(*inst.indexes.get_unchecked(0), *inst.indexes.get_unchecked(1)),
            Opc::Eq  => self.eq_regs(*inst.indexes.get_unchecked(0), *inst.indexes.get_unchecked(1)),

            Opc::Addl => self.add_assign(*inst.indexes.get_unchecked(0), *inst.indexes.get_unchecked(1)),

            Opc::Call => return self.call_fn(
                *inst.indexes.get_unchecked(0),
                &inst.indexes.get_unchecked(1..),
            ),

            Opc::Loop => return self.run_loop(&inst.children.get_unchecked(0)),

            Opc::Ret => return Control::Return(*inst.indexes.get_unchecked(0)),
            Opc::RetScope => return Control::ReturnScope,
            Opc::Continue => return Control::Continue,
            Opc::Break => return Control::Break,
            // Opc::Break => panic!("YES!"),

            other => unimplemented!("opcode {:?} is not implemented", other)
        } };

        return Control::Default;
    }

    pub fn run_all(&mut self, insts: &[Inst]) -> Control {
        for inst in insts.iter() {
            let control = self.run_one(inst);
            match control {
                Control::Default => {},
                other => return other
            }
        }

        return Control::Default;
    }
}
