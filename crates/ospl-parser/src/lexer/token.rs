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
    If,
    Else,
    Loop,
    Use,
    Foreign,

    /* values */
    Nul,
    Undefined,
    True,
    False,

    /* types */
    IntT, FloatT, StrT, BoolT, List,

    /* punctuation */
    Atsign, Comma, Dot, Colon, Semicolon,

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
}

pub struct TokenExpectation {
    pub matches: fn(&Token) -> bool,
    pub label: &'static str,
}

#[macro_export] macro_rules! tExp {
    ($($variant:ident),+ $(,)?) => {{
        fn matches(t: &Token) -> bool {
            matches!(t, $(
                Token::$variant
            )|+)
        }

        TokenExpectation {
            matches,
            label: concat!($(stringify!($variant), " | "),+),
        }
    }};
}

#[macro_export] macro_rules! tComb {
    ($label:expr, $($exp:expr),+ $(,)?) => {
        TokenExpectation {
            matches: |t| {
                false $(|| (($exp).matches)(t))+
            },
            label: $label,
        }
    };
}

pub const EXP_KEYWORD: TokenExpectation =
    tExp!(Fn, Do, Def, Scope, Return, Continue, If, Else, Loop, Use);

pub const EXP_IDENT: TokenExpectation = TokenExpectation {
    matches: |t| -> bool {
        matches!(t, Token::Ident(_))
    },
    label: "Ident"
};

#[derive(Clone, Debug)]
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