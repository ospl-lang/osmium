pub mod token;
pub mod lexer;

#[cfg(test)]
mod tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::Token;

    #[test]
    fn lex_def() {
        let mut l = Lexer::new("def x = 10");
        let spans = l.all_tokens();
        let mut toks = Vec::new();
        for span in spans {
            toks.push(span.destructure().1);
        }

        assert_eq!(
            toks,
            vec![
                Token::Def,
                Token::Ident("x".to_string()),
                Token::Equals,
                Token::Integer(10),
            ]
        );
    }
}