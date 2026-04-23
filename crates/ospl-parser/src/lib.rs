pub mod tests;
mod expr;
mod stmt;
mod literal;
mod file;
mod types;

pub struct Parser<'a> {
    pub input: &'a str,

    /// Must always be a valid UTF8 start in [`Self::input`]
    pub pos: usize
}

#[derive(Clone)]
pub enum Keyword {
    Return,
    Def,
    Fn,
}

pub static ALL_KWS: &[Keyword] = &[Keyword::Def, Keyword::Return];

impl Keyword {
    pub fn as_str(&self) -> &'static str {
        return match self {
            // remember: add a space at the end!
            Self::Return => "return ",
            Self::Def => "def ",
            Self::Fn => "fn ",
        }
    }

    pub fn str_is_keyword(s: &str) -> bool {
        return match s {
            "def" | "return" | "fn" => true,
            _ => false
        }
    }
}

impl<'p> Parser<'p> {
    pub fn new(input: &'p str) -> Self {
        return Self {
            input,
            pos: 0
        }
    }

    pub fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();  // advance cursor
            } else {
                break;
            }
        }
    }

    pub fn attempt<T>(&mut self, f: fn(&mut Self) -> Option<T>) -> Option<T> {
        let start = self.pos;
        if let Some(r) = f(self) {
            return Some(r)
        } else {
            self.pos = start;
            return None
        }
    }

    pub fn keywords_peek(&mut self, options: &'static [Keyword]) -> Option<&Keyword> {
        for option in options {
            if self.input[self.pos..].starts_with(option.as_str()) {
                return Some(&option);
            }
        }

        return None
    }

    pub fn keywords_comsume(&mut self, options: &'static [Keyword]) -> Option<&Keyword> {
        for option in options {
            if self.input[self.pos..].starts_with(option.as_str()) {
                self.pos += option.as_str().len();

                return Some(&option);
            }
        }

        return None
    }

    #[inline(always)]
    pub fn peek(&self) -> Option<char> {
        // Rust zero-cost makes this very fast!
        return self.input[self.pos..].chars().next();
    }

    pub fn read_until<F>(&mut self, closure: F) -> &str
    where
        F: Fn(char) -> bool
    {
        let start = self.pos;

        while let Some(ch) = self.peek() {
            if closure(ch) {
                break;
            }
            self.pos += ch.len_utf8();  // usnure if this is before or after...
        }

        return &self.input[start..self.pos]
    }
}
