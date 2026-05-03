use ospl_common::ast::Position;

use crate::lexer::token::{Span, Token};

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    peeked: Option<char>,
    position: Position,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            peeked: None,
            position: Position::default(),
        }
    }

    fn bump(&mut self) -> Option<char> {
        let p = self.peeked.take().or_else(|| self.chars.next());
        if p == Some('\n') {
            self.position.next_line();
        } else {
            self.position.next_column();
        }

        return p
    }

    fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }
        self.peeked
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.bump();
        }
    }

    fn do_dup(&mut self, c: char, one: Token, two: Token) -> Token {
        if self.peek() == Some(c) {
            self.bump();
            return two
        } else { return one }
    }
}

impl<'a> Lexer<'a> {
    pub fn next_token(&mut self) -> Option<Span> {
        self.skip_ws();
        let pos = self.position;

        let c = self.bump()?;

        let tok = match c {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::LBracket,
            '{' => Token::LSquirly,
            '}' => Token::RSquirly,
            '<' => Token::LAngle,
            '>' => Token::RAngle,

            '+' => Token::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.bump();
                    Token::Arrow
                } else {
                    Token::Dash
                }
            },
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,

            ',' => Token::Comma,
            '.' => Token::Dot,
            '@' => Token::Atsign,
            ';' => Token::Semicolon,
            ':' => Token::Colon,

            '&' => self.do_dup(c, Token::LogicAnd, Token::BitwiseAnd),
            '|' => self.do_dup(c, Token::LogicOr, Token::BitwiseOr),
            '!' => self.do_dup(c, Token::LogicNot, Token::BitwiseNot),

            '=' => self.do_dup(c, Token::Equals, Token::IsEqual),

            c if c.is_ascii_digit() => {
                let mut s = String::new();
                s.push(c);

                while matches!(self.peek(), Some(p) if p.is_ascii_digit()) {
                    s.push(self.bump().unwrap());
                }

                Token::Integer(s.parse().unwrap())
            }

            c if c.is_alphabetic() || c == '_' => {
                let mut s = String::new();
                s.push(c);

                while matches!(self.peek(), Some(p) if p.is_alphanumeric() || p == '_') {
                    s.push(self.bump().unwrap());
                }

                match s.as_str() {
                    /* keywords */
                    "fn" => Token::Fn,
                    "scope" => Token::Scope,
                    "def" => Token::Def,
                    "return" => Token::Return,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "loop" => Token::Loop,
                    "use" => Token::Use,

                    /* types */
                    "int" => Token::IntT,
                    "float" => Token::FloatT,
                    "str" => Token::StrT,
                    "bool" => Token::BoolT,

                    /* values */
                    "nul" => Token::Nul,
                    "undefined" => Token::Undefined,
                    "true" => Token::True,
                    "false" => Token::False,

                    _ => Token::Ident(s),
                }
            }

            '\'' => {
                let mut s = String::new();
                
                loop {
                    let x = self.peek()?;
                    if x == '\'' {
                        break Token::StringLit(s)
                    }
                    self.bump()?;
                    s.push(x);
                }
            }

            _ => panic!("Unexpected character: {c}"),
        };

        Some(Span::new(pos, tok))
    }

    pub fn all_tokens(&mut self) -> Vec<Span> {
        let mut toks = Vec::new();
        while let Some(t) = self.next_token() {
            toks.push(t);
        }

        return toks
    }
}
