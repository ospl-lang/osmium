use arena::{ArenaIndex, Arena};
use ospl_common::{ast::frame::RuntimeFrame, inst::{RuntimeFunction, RuntimeValue, optimized::{Inst, Opc}}, types::AbsAddress};
use crate::arena::ArenaItem;

mod ffi;
mod cond;
mod binaryops;
mod pushes;
mod function;
mod unaryops;
mod list;
mod types;
pub mod arena;
pub mod gc;

pub mod tests;

#[derive(Debug)]
pub struct VM {
    pub arena: Arena,
    pub stack: Vec<RuntimeFrame>,
    pub ffi: ffi::FfiRegistry
}

#[derive(Debug)]
pub enum Control {
    Break,
    Continue,
    Return(AbsAddress),
    ReturnScope,
    Default,
}

impl VM {
    pub fn new() -> Self {
        return Self {
            arena: Arena::new(),
            stack: vec![RuntimeFrame::default()],
            ffi: ffi::FfiRegistry::default(),
        }
    }

    #[inline(always)]
    pub fn push_frame(&mut self, f: RuntimeFrame) {
        self.arena.gc_frame_added(&f.indexes);
        self.stack.push(f);
    }

    #[inline(always)]
    pub fn new_child_of_scope(&mut self, f: &RuntimeFrame) {
        let parents = f.indexes.clone();

        // adding this makes it not behave according to spec
        // self.arena.gc_frame_added(&parents);
        self.stack.push(RuntimeFrame {
            indexes: parents,
            num_args: f.num_args,
            num_captures: f.num_captures,
        });
    }

    #[inline(always)]
    pub fn new_scope_parental(&mut self) {
        let parents = self.top().indexes.clone();

        // adding this makes it not behave according to spec
        // self.arena.gc_frame_added(&parents);

        self.stack.push(RuntimeFrame {
            indexes: parents,
            num_args: self.top().num_args,
            num_captures: self.top().num_captures,
        });
    }

    #[inline(always)]
    pub fn end_scope(&mut self) {
        let Some(f) = self.stack.pop()
            else { panic!("You can't return from the top level of a script, you fucking moron!") };

        self.arena.gc_frame_destroyed(&f.indexes);
    }

    #[inline(always)]
    pub unsafe fn pop_scope_without_gc(&mut self) -> RuntimeFrame {
        let Some(f) = self.stack.pop()
            else { panic!("You can't return from the top level of a script, you fucking moron!") };

        return f
    }

    /// Returns the scope without decrementing the refcount
    #[inline(always)]
    pub fn pop_scope(&mut self) -> RuntimeFrame {
        let f = self.stack.pop()
            .unwrap_or_else(|| panic!("You can't return from the top level of a script, you fucking moron!"));
    
        self.arena.gc_frame_destroyed(&f.indexes);
        return f
    }

    #[inline(always)]
    pub fn get_item_top(&mut self, abs: ArenaIndex) -> &mut ArenaItem {
        return self.arena.get_item_mut(self.top().indexes[abs]);
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
        // It's done to improve performance. Rust adds
        // a bunch of bounds-checking to Vec<T> indexes,
        // but we know that it's safe as long as our
        // instructions are valid. Rust doesn't know
        // this, so we have to tell it ourselves, which
        // requires this unsafe code.
        //
        // TL;DR the safety invariant here is that the
        // instructions are valid.

        unsafe { match &inst.opcode {
            Opc::PushLiteral => {
                self.push_copy(
                    inst.immediate
                        .as_ref()
                        .unwrap()
                );
            },

            Opc::PushArray => self.push_array(&inst.indexes),
            Opc::IndexArray => self.index_array(
                inst.get_index(0),
                inst.get_index(1),
            ),
            Opc::SliceArray => self.slice_array(
                inst.get_index(0),
                inst.get_index(1),
                inst.get_index(2),
            ),

            Opc::QuestionMark => self.question_mark(inst.get_index(0), inst.get_index(1)),

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
            },

            Opc::PushFrame => {
                let idxs: Vec<usize> = inst.indexes.iter().map(|i| self.top().indexes[*i]).collect();
                self.push_literal(RuntimeValue::Scope(RuntimeFrame::new(idxs, 0, 0)));
            },

            Opc::If => return self.if_statement(
                inst.get_index(0),
                &inst.children.get_unchecked(0),
                &inst.children.get_unchecked(1),
            ),

            Opc::AssignCopy => self.assign_copy(inst.get_index(0), inst.get_index(1)),

            Opc::AssignLiteral => {
                // don't even fuck with this one lmao.
                *self.arena.get_mut(self.top().indexes[inst.indexes[0]]) = inst.immediate.as_ref().unwrap().clone();
            },

            Opc::Property => {
                let (x, prop) = (inst.get_index(0), inst.get_index(1));

                // this is just legitimely fucking safe. There's no invariant here.
                let search_in = self.raw_get_value_top(x);
                match &*search_in {
                    RuntimeValue::Scope(s) => {
                        let cutoff = s.num_args + s.num_captures;
                        let indexes = &s.indexes[cutoff..];
                        let add_thing = indexes[prop];

                        // we don't incremenet the refcount because that only
                        // happens at frame boundaries.

                        self.top_mut().indexes.push(add_thing);
                    },
                    _ => std::hint::unreachable_unchecked()
                }
            },

            Opc::Cast => self.cast_value(inst.get_index(0), inst.get_index(1)),

            Opc::GetLength => {
                let x = self.get_value_top(inst.get_index(0));
                let l = x.get_length();
                let l = RuntimeValue::Int(l as i64);
                self.push_literal(l);
            }

            Opc::RetScope => return Control::ReturnScope,

            Opc::Add => self.add_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Sub => self.sub_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Eq  => self.eq_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Neq => self.neq_regs(inst.get_index(0), inst.get_index(1)),
            // Opc::Gt  => self.gt_regs(inst.get_index(0), inst.get_index(1)),
            // Opc::Lt  => self.lt_regs(inst.get_index(0), inst.get_index(1)),
            // Opc::Gte => self.gte_regs(inst.get_index(0), inst.get_index(1)),
            // Opc::Lte => self.lte_regs(inst.get_index(0), inst.get_index(1)),

            Opc::Decrement => self.dec_value(inst.get_index(0)),

            Opc::Addl => self.add_assign(inst.get_index(0), inst.get_index(1)),

            Opc::Call => return self.call_fn(
                inst.get_index(0),
                inst.indexes.get_unchecked(1..),
            ),

            Opc::FFICall => { self.call_foreign_function(inst.get_index(0), &inst.indexes[1..]); },

            Opc::Loop => return self.run_loop(&inst.children.get_unchecked(0)),

            Opc::Ret => return Control::Return(self.top().indexes[inst.get_index(0)]),
            Opc::Continue => return Control::Continue,
            Opc::Break => return Control::Break,
            // FFI stuff

            Opc::FFILoadLib => {
                let RuntimeValue::Str(s) = &*self.raw_get_value_top(inst.get_index(0))
                    else { panic!("expected str for FFI instruction") };

                let l = self.ffi.load_library(s).expect("failed to load FFI");
                self.push_literal(RuntimeValue::ForeignLib(l));
            },

            Opc::FFILoadFn => {
                let RuntimeValue::ForeignLib(lib) = *self.get_value_top(inst.get_index(0))
                    else { unreachable!("no idea what this is") };

                let RuntimeValue::Str(s) = &*self.raw_get_value_top(inst.get_index(1))
                    else { panic!("expected str for FFI instruction") };

                let rtype = ffi::type_number_to_string(inst.get_index(2));

                // do the thing
                let mut types = Vec::new();
                for remaining_index in inst.indexes.get(3..).unwrap() {
                    types.push(ffi::type_number_to_string(*remaining_index as usize).to_string());
                }

                let handle = self.ffi.register_function(lib, s, types, rtype.to_string()).unwrap();
                self.push_literal(RuntimeValue::ForeignFn(handle));
            }

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
