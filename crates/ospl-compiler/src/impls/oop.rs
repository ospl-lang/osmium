use std::collections::HashMap;

use ospl_common::{ast::{class::{CTable, instance::{InstanceItem, InstanceType}}, module::VPath, repr::Type}, inst::unoptimized::VMInstruction};

use crate::{Compiler, base::{CompilerError, Eval, Res}};

impl Compiler {
    pub fn declare_class(
        &mut self,
        vp: &VPath,
        ctable: &CTable,
    ) -> Res<()> {
        self.vroot.create_member(vp, Type::Class(ctable.clone()))?;
        return Ok(())
    }

    pub fn vp_instantiate_class(
        &mut self,
        vp: &VPath,
        out: &mut Vec<VMInstruction>
    ) -> Res<Eval> {
        let Type::Class(ctable) = self.vroot.get_member_mut(vp)?.clone()
            else { return Err(CompilerError::GenericFIXME) };  // FIXME

        return self.do_instantiate_class(&ctable, out);
    }

    pub fn do_instantiate_class(
        &mut self,
        ctable: &CTable,
        out: &mut Vec<VMInstruction>
    ) -> Res<Eval> {
        let mut items = HashMap::new();
        for (name, field) in ctable.fields() {
            items.insert(
                name.to_owned(),
                InstanceItem {
                    index: 0,
                    ty: Box::new(field.typ.to_owned()),
                }
            );
        }

        // for (name, method) in ctable.methods() {
            // method
        // }

        return Ok(Eval { index: 0, ty: Type::Instance(InstanceType {
            values: items
        })});
    }
}
