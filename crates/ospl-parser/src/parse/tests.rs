use ospl_common::ast::Stmt;

use crate::lexer::lexer::Lexer;

use super::Parser;

#[test]
fn definition() {
    let mut lexer = Lexer::new("def x = 10");
    let toks = lexer.all_tokens();
    let mut parser = Parser::new(&toks, "stdin".to_string());
    let _ = parser.parse_stmt().unwrap();
}

/// Regression test: a `type if` (`if# ... { ... }`) placed at the very end of
/// the input must not be silently dropped by the parser.
///
/// Before the fix, the trailing `else`-lookahead called `self.peek()?`, which
/// returns `Err(PE::EOF)` at end of input. That error propagated out of
/// `parse_type_if`, and `parse_tlc` treated it as end-of-input, discarding the
/// statement (and its entire body).
#[test]
fn trailing_type_if_is_not_dropped() {
    let src = "distinct A: int\ndistinct B: int\ndef U: A | B\ndef x = ((42 as A) as U)\ndef y = ((47 as B) as U)\nif# z@: A = x { do z }\nif# z@: B = y { do z }";
    let mut lexer = Lexer::new(src);
    let toks = lexer.all_tokens();
    let mut parser = Parser::new(&toks, "stdin".to_string());
    let stmts = parser.parse_tlc().unwrap();

    let type_ifs: Vec<&Stmt> = stmts
        .iter()
        .filter_map(|s| match &*s.inner {
            Stmt::TypeIf { .. } => Some(&*s.inner),
            _ => None,
        })
        .collect();

    // Both trailing `if#` statements must survive parsing.
    assert_eq!(
        type_ifs.len(),
        2,
        "expected both trailing `if#` statements to be parsed, got {type_ifs:?}"
    );
}