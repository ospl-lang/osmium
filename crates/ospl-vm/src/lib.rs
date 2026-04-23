use arena::{ArenaIndex, Arena};
use crate::{arena::ArenaItem, gc::GcEvent, inst::optimized::{Inst, Opc}, oop::Instance};

mod ffi;
mod cond;
mod binaryops;
mod pushes;
pub mod arena;
pub mod function;
pub mod oop;
pub mod gc;
pub mod list;
pub mod inst;

pub mod tests;

// TODO: experement with this alignment attribute
//#[repr(align(1))]
#[derive(Debug, Clone)]
/// Represents any OSPL value. **PLEASE READ THE THING BELOW!!**
/// 
/// # **THE PartialEq IMPLEMENTATION SHOULD ONLY BE USED IN TESTS!!!**
/// It ONLY determines if the value is EXACTLY eq/ne
/// 
/// # Dev notes
/// Minimize memory use, even if you have to box a field.
pub enum Value {
    /// Represents any illigal interpreter state (e.g. not allocated)
    Nul,
    Undefined,

    List(Box<list::List>),

    Int(i64),
    Float(f64),
    Str(Box<String>),

    Bool(bool),

    Fn(Box<function::Fn>),

    Instance(Box<Instance>),
}

impl Eq for Value {}

impl Value {
    /// Returns the boolean, or `None` if it is not a boolean.
    /// 
    /// # **THIS SHOULD NOT BE USED TO CHECK TRUTHINESS!!!**
    /// USE [`VM::get_truthiness`] for that
    pub fn as_bool(&self) -> Option<bool> {
        return match self {
            Value::Bool(x) => Some(*x),
            Value::Int(x) => Some(if *x != 0 {true} else {false}),
            _ => None
        }
    }

    pub fn exactly_equal(&self, other: &Value) -> bool {
        return match (self, other) {
            (Self::Nul, Self::Nul) => true,
            (Self::Undefined, Self::Undefined) => true,
            (Self::Int(x), Self::Int(y)) => x == y,
            (Self::Float(x), Self::Float(y)) => x != y,
            _ => false
        };
    }

    /// Returns a reference to [`function::Fn`] if `self` is [`Value::Fn`], and
    /// [`None`] otherwise.
    pub fn as_fn(&self) -> Option<&function::Fn> {
        return match self {
            Self::Fn(x) => Some(&x),
            _ => None
        }
    }

    /// Returns if the type is composite or not.
    /// 
    /// Composite types are types that do not have special behaviour when cloned
    pub fn is_composite(&self) -> bool {
        return matches!(self,
            Value::Nul |
            Value::Undefined |
            Value::Bool(_) |
            Value::Float(_) |
            Value::Fn(_) |
            Value::Int(_)
        );
    }

    pub fn clone_composite(&self) -> Self {
        debug_assert!(self.is_composite());

        return self.clone()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        return self.exactly_equal(other)
    }

    fn ne(&self, other: &Self) -> bool {
        return !self.exactly_equal(other)
    }
}

impl Default for Value {
    fn default() -> Self {
        return Self::Undefined;
    }
}

#[derive(Debug, Default)]
/// Represents a frame on the callstack.
/// 
/// A list of [`ArenaIndex`] is used to translate the local values into
/// absolute adresses.
pub struct Frame {
    /// Stores indexes into the arena
    pub indexes: Vec<usize>
}

impl Frame {
    pub fn new(indexes: Vec<usize>) -> Self {
        return Self { indexes };
    }
}

// WE WILL EVENTUALLY SWITCH TO FIXEDVEC. WHEN FIXEDVEC BECOMES
// FAST ENOUGH.
// pub const VM_FRAME_SIZE: usize = 256;

#[derive(Debug)]
pub struct VM {
    pub arena: Arena,
    pub stack: Vec<Frame>,
}

pub enum Control {
    Break,
    Continue,
    Return(ArenaIndex),
    Default,
}

impl VM {
    pub fn new() -> Self {
        return Self {
            arena: Arena::new(),
            stack: vec![Frame::default()],
        }
    }

    pub fn push_frame(&mut self, f: Frame) {
        self.arena.gc_event(GcEvent::FrameAdded {
            indexes: &f.indexes
        });
        self.stack.push(f);
    }

    pub fn new_child_of_scope(&mut self, f: &Frame) {
        let parents = f.indexes.clone();
        self.arena.gc_event(GcEvent::FrameAdded {
            indexes: parents.as_slice()
        });

        self.stack.push(Frame {
            indexes: parents
        });
    }

    pub fn get_item_top(&mut self, abs: ArenaIndex) -> &mut ArenaItem {
        return self.arena.get_item_mut(self.top().indexes[abs]);
    }

    pub fn new_scope_but_parental(&mut self) {
        let parents = self.top().indexes.clone();
        self.arena.gc_event(GcEvent::FrameAdded {
            indexes: parents.as_slice()
        });

        self.stack.push(Frame {
            indexes: parents
        });
    }

    pub fn end_scope(&mut self) {
        let Some(f) = self.stack.pop()
            else { return };

        self.arena.gc_event(GcEvent::FrameDestroyed {
            indexes: f.indexes.as_slice()
        });
    }

    #[inline(always)]
    pub fn top_mut(&mut self) -> &mut Frame {
        return self.stack.last_mut().unwrap()
    }

    #[inline(always)]
    pub fn top(&self) -> &Frame {
        return self.stack.last().unwrap()
    }

    /// Pushes the value onto the next register in the stack
    /// 
    /// Is about equivalent to literal assignemnt.
    #[inline(always)]
    pub fn push_literal(&mut self, v: Value) -> ArenaIndex {
        let i = self.arena.push(v);
        self.top_mut().indexes.push(i);
        return i
    }

    /// Returns a reference to a value given an index, using the top frame's
    /// index array
    fn get_value_top(&self, i: ArenaIndex) -> &Value {
        return self.arena.get(self.top().indexes[i])
    }

    unsafe fn raw_get_value_top(&self, i: ArenaIndex) -> *const Value {
        return unsafe{ self.arena.raw_get(self.top().indexes[i]) }
    }

    unsafe fn raw_get_value_top_mut(&mut self, i: ArenaIndex) -> *mut Value {
        return unsafe{ self.arena.raw_get_mut(self.top().indexes[i]) }
    }

    /// Returns a reference to a value given an index, using the top frame's
    /// index array
    #[allow(dead_code)]
    fn get_mut_value_top(&mut self, i: ArenaIndex) -> &mut Value {
        return self.arena.get_mut(self.top().indexes[i])
    }

    fn assign_copy(&mut self, reg: usize, new: usize) {
        let x = self.get_value_top(new).clone();
        let b = self.get_mut_value_top(reg);
        *b = x;
    }

    pub fn run_one_optimized(&mut self, inst: &Inst) -> Control {
        match &inst.opcode {
            Opc::PushLiteral => {
                self.push_copy(
                    inst.immediate
                        .as_ref()
                        .unwrap()
                );
            },

            Opc::If => return self.if_statement(
                inst.indexes[0],
                &inst.children[0],
                &inst.children[1]
            ),

            Opc::AssignCopy => self.assign_copy(
                inst.indexes[0],
                inst.indexes[1]
            ),

            Opc::AssignLiteral => {
                *self.arena.get_mut(self.top().indexes[inst.indexes[0]]) = inst.immediate.as_ref().unwrap().clone_composite();
            },

            Opc::Add => self.add_regs(inst.indexes[0], inst.indexes[1]),
            Opc::Sub => self.sub_regs(inst.indexes[0], inst.indexes[1]),
            Opc::Eq  => self.eq_regs(inst.indexes[0], inst.indexes[1]),

            Opc::Addl => self.add_assign(inst.indexes[0], inst.indexes[1]),

            Opc::Call => return self.call_fn(
                inst.indexes[0],
                &inst.indexes[1..]
            ),

            Opc::Loop => return self.run_loop(&inst.children[0]),

            Opc::Ret => return Control::Return(inst.indexes[0]),
            Opc::Continue => return Control::Continue,
            Opc::Break => return Control::Break,

            other => unimplemented!("opcode {:?} is not implement", other)
        };

        return Control::Default;
    }

    pub fn run_all(&mut self, insts: &[Inst]) -> Control {
        for inst in insts.iter() {
            match self.run_one_optimized(inst) {
                Control::Default => {},
                other => return other
            }
        }

        return Control::Default;
    }
}
