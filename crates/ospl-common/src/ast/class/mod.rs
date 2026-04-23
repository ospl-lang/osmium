use std::collections::{BTreeMap, HashMap};
use crate::ast::repr::{FunctionData, Type};

pub mod builder;
pub mod instance;

#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CTable {
    methods: HashMap<String, ClassMethod>,
    fields: BTreeMap<String, ClassField>,
}

impl CTable {
    pub fn methods(&self) -> &HashMap<String, ClassMethod> {
        return &self.methods
    }

    pub fn fields(&self) -> &BTreeMap<String, ClassField> {
        return &self.fields
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassMethod {
    inner: FunctionData,
}

impl ClassMethod {
    pub fn function(&self) -> &FunctionData {
        return &self.inner
    }

    pub fn function_mut(&mut self) -> &mut FunctionData {
        return &mut self.inner
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassField {
    pub typ: Type,
}

// #[cfg(test)]
// mod tests {
//     use crate::ast::class::builder::CTableBuilder;

//     #[test]
//     fn merge_ctable() {
//         let mut ct1 = CTableBuilder::default()
//             .build();

//         let ct2 = CTableBuilder::default()
//             .build();

//         ct1.merge(ct2);

//         let expectation = CTableBuilder::default()
//             .build();

//         assert_eq!(ct1, expectation);
//     }
// }