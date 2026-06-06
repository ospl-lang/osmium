use std::{any::Any, fmt::Debug};

use crate::ast::{Expr, Expression, LV, LValue, Literal, Position, Statement, Stmt};

pub trait Spannable: Any + Debug {
    fn get_pos(&self) -> Position;

    fn symbol(&self) -> Option<String>;

    fn user_symbol(&self) -> String {
        return self.symbol().unwrap_or_else(|| "???".to_string())
    }

    // ugly hack to get around some crap
    fn spanned(&self) -> Box<dyn Spannable>;
}

#[derive(Debug, Clone, Copy)]
pub struct UnknownLocation;

impl Spannable for UnknownLocation {
    fn get_pos(&self) -> Position {
        return Position::default()
    }

    fn spanned(&self) -> Box<dyn Spannable> {
        return Box::new(self.clone())
    }

    fn symbol(&self) -> Option<String> {
        return None
    }
}

impl Spannable for Statement {
    fn get_pos(&self) -> Position {
        self.at
    }

    fn symbol(&self) -> Option<String> {
        return Some(match &*self.inner {
            Stmt::Define(d) => d.name.clone(),
            Stmt::Assign(left, _) => left.symbol()?,
            Stmt::Expr(e) => e.symbol()?,
            _ => return None
        })
    }

    fn spanned(&self) -> Box<dyn Spannable> {
        return Box::new(self.clone())
    }
}

impl Spannable for Expression {
    fn get_pos(&self) -> Position {
        self.at
    } 

    fn symbol(&self) -> Option<String> {
        return Some(match &*self.inner {
            Expr::LValue(lv) => lv.symbol()?,
            Expr::Call(l, e) => format!(
                "{}({})",
                l.symbol()?,
                e.iter().filter_map(|e| e.symbol()).collect::<String>()
            ),
            Expr::Literal(l) => match l {
                Literal::Address(a) => a.to_string(),
                Literal::Int(i) => i.to_string(),
                Literal::Float(f) => f.to_string(),
                Literal::Bool(b) => b.to_string(),
                Literal::Char(c) => format!("\"{c}\"C"),
                Literal::List(t, v) => format!("list {t:?} {{ {v:?} }} "),
                Literal::Function(f) => format!("fn {f:?}"),  // TODO make these work better
                Literal::Str(s) => format!("\"{s}\""),
                Literal::Undefined => "undefined".to_string(),
                Literal::Nul => "nul".to_string(),
            },
            _ => return None
        })
    }

    fn spanned(&self) -> Box<dyn Spannable> {
        return Box::new(self.clone())
    }
}

impl Spannable for LValue {
    fn get_pos(&self) -> Position {
        self.at
    }

    fn symbol(&self) -> Option<String> {
        return match &*self.inner {
            LV::Variable(v) => Some(v.clone()),
            LV::Property(p, v) => Some(format!("{}.{v}", p.symbol()?)),
            LV::Slice(p, s, e) => Some(format!("{}:({},{})", p.symbol()?, s.symbol()?, e.symbol()?)),
            LV::Index(p, i) => Some(format!("{}:({})", p.symbol()?, i.symbol()?))
        }
    }

    fn spanned(&self) -> Box<dyn Spannable> {
        return Box::new(self.clone())
    }
}
