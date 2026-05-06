use core::panic;

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

    fn do_0dup(&mut self, c: char, zero: Token, one: Token) -> Token {
        if self.peek() == Some(c) {
            self.bump();
            return one
        } else { return zero }
    }

    fn do_2dup(&mut self, c0: char, c1: char, zero: Token, one_tok0: Token, one_tok1: Token) -> Token {
        if self.peek() == Some(c0) {
            self.bump();
            return one_tok0
        }

        else if self.peek() == Some(c1) {
            self.bump();
            return one_tok1
        }
        
        else { return zero }
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
            ']' => Token::RBracket,
            '{' => Token::LSquirly,
            '}' => Token::RSquirly,
            '<' => self.do_0dup('=', Token::LAngle, Token::LessThanEqual),
            '>' => self.do_0dup('=', Token::RAngle, Token::GreaterThanEqual),

            '+' => self.do_dup(c, Token::Plus, Token::Increment),
            '-' => self.do_2dup('>', '-', Token::Dash, Token::Arrow, Token::Decrement),
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,

            ',' => Token::Comma,
            '.' => Token::Dot,
            '@' => Token::Atsign,
            ';' => Token::Semicolon,
            ':' => Token::Colon,

            '#' => {
                // ignore all the text until a newline
                loop {
                    self.peek()?;
                    if self.bump()? == '\n' {
                        // recurse for data
                        return self.next_token();
                    };
                }
            }

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
                    "do" => Token::Do,
                    "scope" => Token::Scope,
                    "def" => Token::Def,
                    "return" => Token::Return,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "loop" => Token::Loop,
                    "use" => Token::Use,
                    "foreign" => Token::Foreign,

                    /* types */
                    "int" => Token::IntT,
                    "float" => Token::FloatT,
                    "str" => Token::StrT,
                    "bool" => Token::BoolT,
                    "list" => Token::List,

                    /* values */
                    "nul" => Token::Nul,
                    "undefined" => Token::Undefined,
                    "true" => Token::True,
                    "false" => Token::False,

                    _ => Token::Ident(s),
                }
            }

            '\"' => {
                let mut s = String::new();

                loop {
                    let x = self.peek()?;
                    if x == '\"' {
                        self.bump()?;  // consume closing '
                        break
                    }

                    self.bump()?;  // consume character
                    s.push(x);
                }

                return Some(Span::new(pos, Token::StringLit(s)));
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
