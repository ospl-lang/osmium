use ospl_common::{ast::{Expression, RStore, Statement, Type, UType, spanning::Spannable}, inst::optimized::{Inst, InstBuilder, Opc}};

use crate::{CE, Compiler, Res};

impl Compiler {
    pub fn compile_if_stmt(
        &mut self,
        left: &Expression,
        yes: &[Statement],
        no: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        let l = self.eval(left, ob)?;

        let mut ob_yes = Vec::new();
        self.stack.push_parental();
        self.compile_block(yes, &mut ob_yes)?;
        self.stack.pop();

        let mut ob_no = Vec::new();
        self.stack.push_parental();
        self.compile_block(no, &mut ob_no)?;
        self.stack.pop();

        let i = InstBuilder::new()
            .opcode(Opc::If)
            .index(l.address)
            .child(ob_yes)
            .child(ob_no)
            .build();
    
        ob.push(i);
        return Ok(())
    }

    pub fn compile_type_if_stmt(
        &mut self,
        left: &Expression,
        union_: &UType,
        ident: &String,
        yes: &[Statement],
        no: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        let _span = tracing::debug_span!("typeif");
        let _enter = _span.enter();

        let unwrap_type = self.rt(self.stack.top(), &union_, left)?;
        let l = self.eval(left, ob)?;

        let lty = l.ty.clone();
        let Type::SafeUnion(lty_alternatives) = &lty
            else { todo!("unwrap - error"); };

        let Some(Type::Nominal(lty_alt_id, _lty_alt_base)) = lty_alternatives.iter().find(|x| **x == unwrap_type)
        else {
            return Err(CE {
                at: left.spanned(),
                during: "type if - type check",
                error: crate::CEData::UnionDoesntHaveType { union: lty.clone(), doesnt_have: unwrap_type },
                msg: None
            })
        };

        // YES BLOCK
        // 
        // This one is special, we need to re-declare the variable under a
        // different ID and type in this new scope.
        let mut ob_yes = Vec::new();
        self.stack.push_parental();

        self.stack.top_mut().direct_declare(
            ident.clone(),
            ospl_common::ast::Entry::Runtime(RStore {
                address: l.address,
                typ: unwrap_type
            })
        );

        self.compile_block(yes, &mut ob_yes)?;

        tracing::error!("yes = {ob_yes:?} source = {yes:?}");

        self.stack.pop();

        // NO BLOCK
        let mut ob_no = Vec::new();
        self.stack.push_parental();
        self.compile_block(no, &mut ob_no)?;
        self.stack.pop();

        // EMIT INSTRUCTION
        let i = InstBuilder::new()
            .opcode(Opc::UIf)
            .index(l.address)
            .index(*lty_alt_id)
            .child(ob_yes)
            .child(ob_no)
            .build();

        tracing::error!("inst = {i:?}");
    
        ob.push(i);
        return Ok(())
    }

    pub fn compile_loop_stmt(
        &mut self,
        inner: &[Statement],
        ob: &mut Vec<Inst>
    ) -> Res<()>
    {
        self.stack.push_parental();
        let mut loop_ob = Vec::new();
        for s in inner {
            self.compile_stmt(s, &mut loop_ob)?;
        }

        self.stack.pop();
        let i = InstBuilder::new()
            .opcode(Opc::Loop)
            .child(loop_ob)
            .build();

        ob.push(i);
        return Ok(())
    }
}
