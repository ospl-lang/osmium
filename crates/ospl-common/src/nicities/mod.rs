use std::fmt::{self, Display, Formatter};
use std::fmt::Write;
use crate::ast::{Entry, Type, UType};

fn write_indent(f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
    for _ in 0..indent {
        f.write_str("    ")?; // 4 spaces
    }
    Ok(())
}

fn fmt_utype(t: &UType, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
    match t {
        UType::Apply(a, b) => {
            fmt_utype(a, f, indent)?;
            write!(f, "[")?;

            let mut first = true;
            for (left, right) in b {
                if !first {
                    write!(f, ", ")?;
                }
                first = false;
                write!(f, "{left}: ")?;
                fmt_utype(right, f, indent)?;
            }

            write!(f, "]")?;
        }

        UType::Function(fun) => {
            writeln!(f, "fn(")?;

            for arg in &fun.args {
                write_indent(f, indent + 1)?;
                fmt_utype(arg, f, indent + 1)?;
                writeln!(f, ",")?;
            }

            write_indent(f, indent)?;
            write!(f, ") -> ")?;
            fmt_utype(&fun.ret, f, indent)?;
        }

        UType::InferScope => write!(f, "scope")?,
        UType::List(l) => {
            write!(f, "list ")?;
            fmt_utype(l, f, indent)?;
        }
        UType::Nominal(base) => {
            write!(f, "let ")?;
            fmt_utype(base, f, indent)?;
        }
        UType::Property(a, b) => write!(f, "{a}.{b}")?,
        UType::Resolved(r) => write!(f, "{r:?}")?,
        UType::Returnof(r) => write!(f, "@{r}")?,
        UType::Scope(s) => write!(f, "{s:?}")?,
        UType::Typeof(t) => write!(f, "{t}")?,
        UType::Union(a, b) => {
            fmt_utype(a, f, indent)?;
            write!(f, " | ")?;
            fmt_utype(b, f, indent)?;
        }
    }

    Ok(())
}

impl Display for UType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt_utype(self, f, 0)
    }
}

fn indent(buf: &mut String, indent: usize) {
    for _ in 0..indent {
        buf.push_str("    ");
    }
}

fn fmt_type_inner(t: &Type, out: &mut String, indent_level: usize) {
    match t {
        Type::List(inner) => {
            out.push_str("list ");
            fmt_type_inner(inner, out, indent_level);
        }

        Type::Function(f) => {
            out.push_str("fn(\n");

            for arg in &f.args {
                indent(out, indent_level + 1);
                fmt_type_inner(arg, out, indent_level + 1);
                out.push_str("\n");
            }

            for (name, arg) in &f.named_args {
                indent(out, indent_level + 1);
                out.push_str("def ");
                out.push_str(name);
                out.push_str(": ");
                fmt_type_inner(arg, out, indent_level + 1);
                out.push_str(",\n");
            }

            indent(out, indent_level);
            out.push_str(") -> ");
            fmt_type_inner(&f.ret, out, indent_level);
        }

        Type::Scope(scope) => {
            out.push_str("scope(\n");

            let mut entries: Vec<_> = scope.get_inner().iter().collect();
            entries.sort_by_key(|(k, _)| k.as_bytes());

            for (id, ent) in entries {
                indent(out, indent_level + 1);
                out.push_str(id);
                out.push_str(": ");

                match ent {
                    Entry::Alias(a) => fmt_type_inner(&a.typ, out, indent_level + 1),
                    Entry::Runtime(r) => fmt_type_inner(&r.typ, out, indent_level + 1),
                }

                out.push('\n');
            }

            indent(out, indent_level);
            out.push(')');
        }

        Type::Union(a, b) => {
            fmt_type_inner(a, out, indent_level);
            out.push_str(" | ");
            fmt_type_inner(b, out, indent_level);
        }

        Type::Nominal(_id, inner) => {
            let _ = write!(out, "let {_id} ");
            fmt_type_inner(inner, out, indent_level);
        }

        Type::Address => out.push_str("addr"),
        Type::Int => out.push_str("int"),
        Type::Float => out.push_str("float"),
        Type::Any => out.push_str("any"),
        Type::Bool => out.push_str("bool"),
        Type::Char => out.push_str("char"),
        Type::Nul => out.push_str("nul"),
        Type::Undefined => out.push_str("undefined"),
        Type::Str => out.push_str("str"),
        other => {
            let _ = write!(out, "{other:?}");
        }
    }
}

pub fn fmt_type(t: Type) -> String {
    let mut out = String::new();
    fmt_type_inner(&t, &mut out, 0);
    out
}