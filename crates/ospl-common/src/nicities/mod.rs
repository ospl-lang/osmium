use std::fmt::Display;

use crate::ast::UType;

impl Display for UType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apply(a, b) => {
                a.fmt(f)?;
                write!(f, "[")?;
                for (left, right) in b {
                    write!(f, "{left}: {right}")?;
                }
                write!(f, "]")?;
            },
            Self::Function(fun) => {
                write!(f, "fn(")?;
                for arg in &fun.args {
                    write!(f, "{arg}")?;
                }

                write!(f, ") -> ")?;
                fun.ret.fmt(f)?;
            },
            Self::InferScope => {
                write!(f, "scope")?;
            },
            Self::List(l) => {
                write!(f, "list {l}")?;
            },
            Self::Nominal(base) => {
                write!(f, "let {base}")?;
            },
            Self::Property(a, b) => {
                write!(f, "{a}.{b}")?;
            },
            Self::Resolved(r) => {
                write!(f, "{r:?}")?;
            },
            Self::Returnof(r) => {
                write!(f, "@{r}")?;
            },
            Self::Scope(s) => {
                write!(f, "{s:?}")?;
            },
            Self::Typeof(t) => {
                write!(f, "{t}")?;
            },
            Self::Union(a, b) => {
                write!(f, "{a} | {b}")?;
            },
        }

        Ok(())
    }
}