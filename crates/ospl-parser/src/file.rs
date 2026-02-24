use ospl_common::ast::repr::Stmt;

use crate::Parser;

impl<'a> Parser<'a> {
    pub fn parse_stmt_list(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while let Some(stmt) = self.parse_stmt() {
            statements.push(stmt);
            self.skip_ws();
        }

        return statements;
    }

    pub fn parse_file(&mut self) -> ParserFile {
        return ParserFile {
            statements: self.parse_stmt_list()
        }
    }
}

pub struct ParserFile {
    pub statements: Vec<Stmt>
}