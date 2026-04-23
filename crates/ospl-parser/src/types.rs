use ospl_common::ast::repr::{FunctionType, Parameter, Type};

use crate::{Keyword, Parser};

impl<'a> Parser<'a> {
    pub const TYPE_KEYWORDS: &'static [Keyword] = &[
        Keyword::Fn,
    ];

    pub fn parse_function_type(&mut self) -> Option<FunctionType> {
        // argument list
        self.expect_char('(')?;
        let mut args = Vec::new();

        loop {
            // id: type
            let Some(ident) = self.parse_ident()
                else { break };

            self.skip_ws();
            self.expect_char(':')?;  // don't remove question mark
            self.skip_ws();
            let ty = self.parse_type()?;
            self.skip_ws();

            args.push(Parameter {
                ident, ty
            });

            // separator
            if self.peek_or_consume(',') { break }
            self.skip_ws();
        };

        self.expect_char(')')?;

        return None
    }

    pub fn parse_type(&mut self) -> Option<Type> {
        let kw = self.keywords_comsume(Self::TYPE_KEYWORDS)?;
        return Some(match kw {
            Keyword::Fn => Type::Function(self.parse_function_type()?),
            _ => unimplemented!(),
        })
    }
}