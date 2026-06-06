use crate::lexer::lexer::Lexer;

use super::Parser;

#[test]
fn definition() {
    let mut lexer = Lexer::new("def x = 10;");
    let toks = lexer.all_tokens();
    let mut parser = Parser::new(&toks);
    let _ = parser.parse_stmt().unwrap();
}
