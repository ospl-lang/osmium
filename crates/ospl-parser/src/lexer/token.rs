use std::fmt::Debug;

use ospl_common::ast::Position;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    /* keywords */
    Def,
    Let,
    Do,
    Fn,
    Scope,
    Return,
    Break,
    Continue,
    Select,
    If,
    Else,
    Loop,
    Use,
    Foreign,
    For,
    While,
    As,
    Try,
    Copy,

    /* macro stuff */
    Macro,
    DollarSign,

    /* values */
    Nul,
    Undefined,
    True,
    False,

    /* types */
    IntT, FloatT, StrT, BoolT, ListT, AddrT, UnknownT, AnyT,

    /* punctuation */
    Atsign, Comma, Dot, Ellipsis, Colon, Semicolon, Question,

    /// `->` symbol
    Arrow,

    Plus, Dash, Star, Slash, Percent,

    /// `&` symbol
    LogicAnd,
    
    /// `&&` symbol
    BitwiseAnd,

    /// `|` symbol
    LogicOr,

    /// `||` symbol
    BitwiseOr,

    /// `!` symbol
    LogicNot,

    /// `!!` symbol
    BitwiseNot,

    /// `^` symbol
    LogicalXor,

    /// `^^` symbol
    BitwiseXor,

    /// `--`
    Decrement,

    /// `++`
    Increment,

    LParen,   RParen,
    LSquirly, RSquirly,
    
    /** `[` symbol */ LBracket,
    /** `]` symbol */ RBracket,
    /** `<` symbol */ LAngle,
    /** `>` symbol */ RAngle,

    /** `=` symbol */  Equals,
    /** `==` symbol */ IsEqual,
    /** `!=` symbol */ IsNotEqual,
    /** `<=` symbol */ LessThanEqual,
    /** `>=` symbol */ GreaterThanEqual,

    /* literals */
    Integer(i64),
    AddressLiteral(u64),
    Float(f64),
    StringLit(String),

    /* other */
    Ident(String),

    Char(char),
    CharT,
}

pub struct TokenExpectation {
    pub matches: Box<dyn Fn(&Token) -> bool>,
    pub label: &'static str,
}

impl Debug for TokenExpectation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.label)
    }
}

#[macro_export] macro_rules! tExp {
    ($($variant:ident),+ $(,)?) => {{
        fn matches(t: &$crate::lexer::token::Token) -> bool {
            matches!(t, $(
                $crate::lexer::token::Token::$variant
            )|+)
        }

        $crate::lexer::token::TokenExpectation {
            matches: Box::new(matches),
            label: concat!($(stringify!($variant), " | "),+),
        }
    }};
}

#[macro_export] macro_rules! tComb {
    ($label:expr, $($exp:expr),+ $(,)?) => {
        $crate::lexer::token::TokenExpectation {
            matches: Box::new(|t| {
                false $(|| (($exp).matches)(t))+
            }),
            label: $label,
        }
    };
}

pub fn exp_keyword() -> TokenExpectation {
    return tExp!(Fn, Do, Def, Let, Scope, Return, Break, Continue, For, While, If, Else, Loop, Use)
}

pub fn exp_ident() -> TokenExpectation {
    TokenExpectation {
        matches: Box::new(|t| -> bool {
            matches!(t, Token::Ident(_))
        }),
        label: "Ident"
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Span(Position, Token);

impl Span {
    pub fn new(pos: Position, tok: Token) -> Self {
        return Self(pos, tok)
    }

    pub fn token(&self) -> &Token {
        return &self.1
    }

    pub fn destructure(self) -> (Position, Token) {
        return (self.0, self.1)
    }

    pub fn position(&self) -> &Position {
        return &self.0
    }
}