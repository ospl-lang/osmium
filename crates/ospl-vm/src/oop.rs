//! Object-oriented services

use crate::{VM, Value, arena::ArenaIndex};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Instance {
    /// A list of indexes referring to the object's members
    members: Vec<ArenaIndex>,
}

impl VM {
    /// Creates a new structure, copying the arguments into the structure. Then
    /// the structure is pushed onto the current frame.
    /// 
    /// This means the caller is left with their own copies that don't affect
    /// the struct's values. This can be confusing.
    pub fn class_constructor(&mut self, args: &[ArenaIndex]) {
        // we must copy this, of course..
        let mut copied_args = Vec::with_capacity(args.len());
        for arg in args {
            let cp = self.get_value_top(*arg).clone();
            let idx = self.arena.push(cp);
            copied_args.push(idx);
        }

        let stru = Instance {
            members: copied_args
        };

        self.push_literal(Value::Instance(Box::new(stru)));
    }

    /// Preforms property access on an object `a` with key `b` in the
    /// current scope.
    /// 
    /// Note that `b` is double-defered
    pub fn property_access_ref(&mut self, a: ArenaIndex, b: ArenaIndex) {
        let a = self.get_value_top(a);
        let Value::Instance(i) = a
            else { panic!("can't property access on a non-instance") };

        if let Value::Int(defer) = self.get_value_top(b) {
            let actual = *defer as usize;
            let idx = i.members[actual];
            self.top_mut().indexes.push(idx);
        } else {
            panic!("please use an Int(...) for memeber indexes!")
        }
    }

    pub fn property_access_static(&mut self, a: ArenaIndex, b: ArenaIndex) {
        let a = self.get_value_top(a);
        let Value::Instance(i) = a
            else { panic!("can't property access on a non-instance") };

        let idx = i.members[b];
        self.top_mut().indexes.push(idx);
    }
}