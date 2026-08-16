use arena::{ArenaIndex, Arena};
use ospl_common::{inst::{RT, RuntimeFunction, RuntimeValue, make, optimized::{Inst, Opc}}, types::AbsAddress};
use crate::{arena::ArenaItem, stack::Stack};

mod ffi;
mod cond;
mod binaryops;
mod function;
mod unaryops;
mod list;
mod types;
pub mod stack;
pub mod debug;
pub mod arena;
pub mod gc;

pub mod tests;

#[derive(Debug)]
pub struct VM {
    pub arena: Arena<RuntimeValue>,
    pub stack: stack::Stack,
    pub ffi: ffi::FfiRegistry,
    pub gc: gc::GC,
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
            arena: Arena::new(None),
            stack: Stack::default(),
            ffi: ffi::FfiRegistry::default(),
            gc: gc::GC::default()
        }
    }

    #[inline(always)]
    pub fn get_item_top_mut(&mut self, rel: ArenaIndex) -> &mut ArenaItem<RuntimeValue> {
        // return self.arena.get_item_mut(self.top().indexes[rel]);
        return self.arena.get_item_mut(self.stack.top_indexes()[rel])
    }

    #[inline(always)]
    pub fn get_item_top(&self, rel: ArenaIndex) -> &ArenaItem<RuntimeValue> {
        // return self.arena.get_item(self.top().indexes[abs]);
        return self.arena.get_item(self.stack.top_indexes()[rel])
    }

    /// Returns a reference to a value given an index, using the top frame's
    /// index array
    #[inline(always)]
    fn get_value_top(&self, rel: ArenaIndex) -> &RuntimeValue {
        return &self.arena.get_item(self.stack.top_indexes()[rel]).inner.as_ref().unwrap()
    }

    #[inline(always)]
    unsafe fn raw_get_value_top(&self, rel: ArenaIndex) -> *const RuntimeValue {
        return unsafe{ self.arena.raw_get(self.stack.top_indexes()[rel]) }
    }

    #[inline(always)]
    unsafe fn raw_get_value_top_mut(&mut self, rel: ArenaIndex) -> *mut RuntimeValue {
        return unsafe{ self.arena.raw_get_mut(self.stack.top_indexes()[rel]) }
    }

    /// Pushes the value onto the next register in the stack
    /// 
    /// Is about equivalent to literal assignemnt.
    #[inline(always)]
    pub fn push_literal(&mut self, v: RuntimeValue) -> ArenaIndex {
        // const MEM_THRES: usize = ((0.6 * arena::MEMMAX as f32) as usize).next_power_of_two();
        // const MEM_THRES: usize = 0b0000001100000000;

        let i = self.arena.push(v);
        self.stack.top_add_index(i);

        let mut gc_retry = 1;
        while (i >> 8 & 0x3) == 0x3 {  // 768
            if self.gc_step(1) {
                break;
            }

            if gc_retry == 6 { break; }
            gc_retry += 1;
        }

        return i
    }

    /// Returns a reference to a value given an index, using the top frame's
    /// index array
    #[allow(dead_code)]
    fn get_mut_value_top(&mut self, i: ArenaIndex) -> &mut RuntimeValue {
        return self.arena.get_mut(self.stack.top_indexes()[i])
    }

    fn assign_ref(&mut self, reg: usize, new: usize) {
        self.stack.top_indexes_mut()[reg] = self.stack.top_indexes()[new];
    }

    #[inline(always)]
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

        #[cfg(debug_assertions)]
        let _dbg = debug::DbgMark::new(format!("run {inst:?}"));

        unsafe { match &inst.opcode {
            Opc::PushLiteral => {
                self.push_literal(
                    inst.immediate
                        .as_ref()
                        .unwrap()
                        .clone()
                );
            },

            Opc::PushArray => self.push_array(&inst.indexes),
            Opc::Index => self.index(
                inst.get_index(0),
                inst.get_index(1),
            ),
            Opc::Slice => self.slice(
                inst.get_index(0),
                inst.get_index(1),
                inst.get_index(2),
            ),

            Opc::PushFunction => {
                let new_indexes = inst.indexes.iter().map(|x| {
                    let idx = self.stack.top_indexes()[*x];

                    return idx;
                }).collect();
                let f_code = inst.children.get_unchecked(0);
                
                let f = RuntimeFunction {
                    captures: new_indexes,
                    code: f_code.clone()
                };

                self.push_literal(make::func(f));
                return Control::Default
            },

            Opc::If => return self.if_statement(
                inst.get_index(0),
                &inst.children.get_unchecked(0),
                &inst.children.get_unchecked(1),
            ),

            Opc::AssignRef => self.assign_ref(inst.get_index(0), inst.get_index(1)),

            Opc::AssignLiteral => {
                // don't even fuck with this one lmao.
                *self.arena.get_mut(self.stack.top_indexes()[inst.indexes[0]]) = inst.immediate.as_ref().unwrap().clone();
            },

            Opc::Property => {
                let (x, prop) = (inst.get_index(0), inst.get_index(1));

                let search_in = self.get_value_top(x);

                // this however isn't
                match search_in.tag {
                    RT::Scope => {
                        let add_thing = search_in.data.scope.indexes[prop];
                        // println!("{add_thing:?} {:?}", self.arena.get(add_thing));

                        self.stack.top_add_index(add_thing);
                    },

                    // _ => std::hint::unreachable_unchecked()
                    other => unimplemented!("cannot do access on value {other:?}"),
                }
            },

            Opc::Cast => self.cast_value(inst.get_index(0), inst.get_index(1)),

            Opc::GetLength => {
                let x = self.get_value_top(inst.get_index(0));
                let l = x.get_length();
                let l = make::addr(l as u64);
                self.push_literal(l);
            }

            Opc::RetScope => return Control::ReturnScope,

            // math
            Opc::Add => self.add_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Sub => self.sub_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Mul => self.mul_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Div => self.div_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Mod => self.mod_regs(inst.get_index(0), inst.get_index(1)),

            // bitwise
            Opc::And => self.and_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Or  => self.or_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Xor => self.xor_regs(inst.get_index(0), inst.get_index(1)),
            Opc::LShift => self.lshift_regs(inst.get_index(0), inst.get_index(1)),
            Opc::RShift => self.rshift_regs(inst.get_index(0), inst.get_index(1)),

            // comparison
            Opc::Eq  => self.eq_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Neq => self.neq_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Gt  => self.gt_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Lt  => self.lt_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Gte => self.gte_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Lte => self.lte_regs(inst.get_index(0), inst.get_index(1)),
            Opc::Neg => self.neg_reg(inst.get_index(0)),

            Opc::Copy => self.copyof(inst.get_index(0)),

            Opc::UWrap => {
                let value = inst.get_index(0);
                let tag = inst.get_index(1);

                self.push_literal(make::union(tag as u64, value));
            },

            Opc::UIf => {
                let value = inst.get_index(0);
                let tag = inst.get_index(1);

                let yes = inst.children.get_unchecked(0);
                let no = inst.children.get_unchecked(1);

                let v = self.get_mut_value_top(value);
                let truth = if let Some(x) = ospl_common::inst::assume::union(v) {
                    let got_tag = x.0;
                    tag as u64 == got_tag
                } else { panic!("UIf() ran on a non-union. stackidx={value:?}, expectedtag={tag:?}, value={v:?}") };

                return if truth {
                    self.run_in_parental(yes)
                } else {
                    self.run_in_parental(no)
                }
            }

            Opc::Decrement => self.dec_value(inst.get_index(0)),
            Opc::Increment => self.inc_value(inst.get_index(0)),

            Opc::Addl => self.add_assign(inst.get_index(0), inst.get_index(1)),
            Opc::Subl => self.sub_assign(inst.get_index(0), inst.get_index(1)),

            Opc::Call => return self.call_fn(
                inst.get_index(0),
                inst.indexes.get_unchecked(1..),
            ),

            Opc::FFICall => { self.call_foreign_function(inst.get_index(0), &inst.indexes[1..]); },

            Opc::Loop => return self.run_loop(&inst.children.get_unchecked(0)),

            Opc::Ret => return Control::Return(self.stack.top_indexes()[inst.get_index(0)]),
            Opc::Continue => return Control::Continue,
            Opc::Break => return Control::Break,
            // FFI stuff

            Opc::FFILoadLib => {
                let Some(s) = (&*self.raw_get_value_top(inst.get_index(0))).as_str()
                    else { panic!("expected str for FFI instruction") };

                let lib = self.ffi.load_library(s).expect("failed to load FFI");

                // check if it has an OSPL_Load function
                if let Ok(func) = self.ffi.register_function(
                    lib,
                    "OSPL_load",
                    vec![
                        "ptr".to_string(),  // pointer to OSPL VM
                    ],
                    "void".to_string()
                ) {
                    let f = self.ffi.get_function(func).unwrap();
                    f.cif.call(f.symbol_ptr, &[
                        libffi::middle::arg(&&raw const self),
                    ])
                }

                self.push_literal(make::foreignlib(lib));
            },

            Opc::FFILoadFn => {
                let lib = self.get_value_top(inst.get_index(0)).assume_foreign_element();

                let Some(s) = (&*self.raw_get_value_top(inst.get_index(1))).as_str()
                    else { panic!("expected str for FFI instruction") };

                let rtype = ffi::type_number_to_string(inst.get_index(2));

                // do the thing
                let mut types = Vec::new();
                for remaining_index in inst.indexes.get(3..).unwrap() {
                    types.push(ffi::type_number_to_string(*remaining_index as usize).to_string());
                }

                let handle = self.ffi.register_function(lib, s, types, rtype.to_string()).unwrap();
                self.push_literal(make::foreignfun(handle));
            },

            other => unimplemented!("{other:?}")
        } };

        return Control::Default;
    }

    #[inline(always)]
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
