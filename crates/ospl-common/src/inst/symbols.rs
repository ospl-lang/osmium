//! Debug symbols

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DebugSymbolTable {
    messages: HashMap<usize, String>,
    inverse: HashMap<String, usize>,
    next_id: usize
}

impl DebugSymbolTable {
    pub fn get_or_add(&mut self, message: String) -> usize {
        if let Some(x) = self.inverse.get(&message) {
            return *x
        } else {
            self.inverse.insert(message.clone(), self.next_id);
            self.messages.insert(self.next_id, message.clone());
            return self.next_id
        }
    }
}