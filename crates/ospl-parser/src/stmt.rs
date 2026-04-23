use crate::{ALL_KWS, Keyword, Parser};
use ::ospl_common::ast::repr::Stmt;

impl<'a> Parser<'a> {
    /// This function is a little weird, basically, it's for ergenomics.
    /// 
    /// Instead of doing `if !self.expect_char('a') { return None }`, you can
    /// just write `self.expect_char('a')?;`
    pub fn expect_char(&mut self, c: char) -> Option<()> {
        let x = self.peek()?;

        if x == c {
            self.pos += x.len_utf8();
            return Some(())
        }

        return None
    }

    pub fn peek_or_consume(&mut self, c: char) -> bool {
        let x = self.peek().expect("unexpected EOF");

        if x == c {
            self.pos += x.len_utf8();
            return true
        }

        return false
    }

    pub fn parse_define_stmt(&mut self) -> Option<Stmt> {
        // now we can parse the name
        let lvalue = self.parse_lvalue()?;

        self.skip_ws();

        let mut rvalue = None;
        if self.peek_or_consume('=') {
            self.skip_ws();
            rvalue = Some(self.parse_expr()?);
        }

        return Some(Stmt::DeclareReal {
            lhs: lvalue,
            rhs: rvalue,
        })
    }

    pub fn parse_return_stmt(&mut self) -> Option<Stmt> {
        return None
    }

    pub fn parse_assignment(&mut self) -> Option<Stmt> {
        let Some(lhs) = self.attempt(Self::parse_lvalue)
            else { return None };

        self.skip_ws();    
        self.expect_char('=')?;
        self.skip_ws();
        
        let Some(rhs) = self.parse_expr()
            else { return None; };  

        return Some(Stmt::AssignReal { lhs, rhs })
    }

    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        // check assignments
        if let Some(assign) = self.attempt(Self::parse_assignment) {
            return Some(assign)
        }

        let kw = self.keywords_comsume(ALL_KWS);
        let stmt = match kw {
            Some(Keyword::Def) => self.parse_define_stmt(),
            Some(Keyword::Return) => self.parse_return_stmt(),
            _ => None
        };

        if stmt.is_some() {
            self.skip_ws();
            self.expect_char(';')?;
            return stmt
        } else { return None }
}
}
