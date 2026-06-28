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

        self.position.ch += 1;

        return p
    }

    fn say_next_token(&mut self) {
        self.position.token_num += 1;
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
            '-' => {
                match self.peek()? {
                    '-' => {
                        self.bump()?;
                        Token::Decrement
                    },

                    '>' => {
                        self.bump()?;
                        Token::Arrow
                    },

                    // negative numbers
                    c if c.is_ascii_digit() => {
                        self.bump()?;
                        self.do_number(c)
                    },

                    _ => Token::Dash
                }
            }
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,

            '^' => if self.peek()? == '^' { Token::BitwiseXor } else { Token::LogicalXor },

            ',' => Token::Comma,
            '.' => if self.peek()? == '.' {
                self.bump()?;
                Token::Ellipsis
            } else { Token::Dot }

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
            },

            '?' => Token::Question,

            '&' => self.do_dup(c, Token::LogicAnd, Token::BitwiseAnd),
            '|' => self.do_dup(c, Token::LogicOr, Token::BitwiseOr),
            '!' => {
                match self.peek()? {
                    '=' => {
                        self.bump()?;
                        Token::IsNotEqual
                    },
                    '!' => {
                        self.bump()?;
                        Token::BitwiseNot
                    },
                    _ => Token::LogicNot
                }
            },
                                                                        
            '=' => self.do_dup(c, Token::Equals, Token::IsEqual),

            c if c.is_ascii_digit() => self.do_number(c),

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
                    "let" => Token::Let,
                    "return" => Token::Return,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "loop" => Token::Loop,
                    "use" => Token::Use,
                    "foreign" => Token::Foreign,
                    "for" => Token::For,
                    "while" => Token::While,
                    "as" => Token::As,
                    "try" => Token::Try,
                    "copy" => Token::Copy,
                    "select" => Token::Select,
                    "macro" => Token::Macro,

                    /* types */
                    "int" => Token::IntT,
                    "float" => Token::FloatT,
                    "str" => Token::StrT,
                    "bool" => Token::BoolT,
                    "list" => Token::ListT,
                    "addr" => Token::AddrT,
                    "unknown" => Token::UnknownT,
                    "any" => Token::AnyT,
                    "char" => Token::CharT,

                    /* values */
                    "nul" => Token::Nul,
                    "undefined" => Token::Undefined,
                    "true" => Token::True,
                    "false" => Token::False,

                    _ => Token::Ident(s),
                }
            }

            '\"' => {
                let s = self.do_str()?;
                if self.peek()? == 'C' {
                    if s.len() != 1 {
                        println!("char literal cannot have less or more than one character");
                        return None;
                    }
                    self.bump()?;
                    Token::Char(s.chars().nth(0).unwrap())
                } else {
                    Token::StringLit(s)
                }
            },

            '$' => Token::DollarSign,

            _ => panic!("Unexpected character: {c}"),
        };

        self.say_next_token();

        Some(Span::new(pos, tok))
    }

    fn do_str(&mut self) -> Option<String> {
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

        return Some(s)
    }

    fn do_number(&mut self, starting: char) -> Token {
        let mut s = String::new();
        s.push(starting);

        while matches!(self.peek(), Some(p) if p.is_ascii_digit()) {
            s.push(self.bump().unwrap());
        }

        if let Some('@') = self.peek() {
            self.bump().expect("failed to bump");
            return Token::AddressLiteral(s.parse().unwrap())
        }

        return Token::Integer(s.parse().unwrap())
    }

    pub fn all_tokens(&mut self) -> Vec<Span> {
        let mut toks = Vec::new();
        while let Some(t) = self.next_token() {
            toks.push(t);
        }

        return toks
    }
}
