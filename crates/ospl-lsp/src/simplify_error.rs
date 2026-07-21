use ospl_compiler::{CE, ast::{Entry, Type}};

pub fn fmt_type(t: Type) -> String {
    match t {
        Type::List(t) => format!("list {}", fmt_type(*t.clone())),
        Type::Address => "addr".to_string(),
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::Any => "any".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Char => "char".to_string(),
        Type::Function(f) => {
            let mut s = "fn(".to_string();
            for arg in f.args {
                s.push_str(&fmt_type(arg));
            }
            s.push(')');

            s.push_str("-> ");
            s.push_str(&fmt_type(f.ret));

            s
        },
        Type::Nul => "nul".to_string(),
        Type::Undefined => "undefined".to_string(),
        Type::Scope(s) => {
            let mut s2 = "scope(".to_string();
            for (id, ent) in s.into_inner() {
                s2.push_str(&format!("{id}: {}", match ent {
                    Entry::Alias(a) => format!("alias {}", fmt_type(a.typ)),
                    Entry::Runtime(r) => format!("value {}", fmt_type(r.typ))
                }));
                s2.push_str(", ");
            }

            s2.push(')');

            s2
        },
        Type::Union(a, b) => {
            format!("{} | {}", fmt_type(*a), fmt_type(*b))
        },
        Type::Str => "str".to_string(),
        Type::Nominal(_, x) => {
            format!("let {}", fmt_type(*x))
        }
        other => format!("{other:?}"),
    }
}

pub fn simplify(e: &CE) -> String {
    match &e.error {
        ospl_compiler::CEData::Bug => "A bug in OSPL was triggered".to_string(),
        ospl_compiler::CEData::InternalError(_) => "An internal error occured".to_string(),
        ospl_compiler::CEData::InvalidAssignOp { op } => {
            format!("Invalid assign operation {op:?}")
        },
        ospl_compiler::CEData::MismatchedTypes { expected, got } => {
            let exp = match expected {
                ospl_compiler::TypeExpectation::AnyList => "list of any type".to_string(),
                ospl_compiler::TypeExpectation::AnyScope => "scope of any type".to_string(),
                ospl_compiler::TypeExpectation::Exact(e) => fmt_type(e.clone()),
                ospl_compiler::TypeExpectation::Indexable => "indexable types (list, str)".to_string(),
                ospl_compiler::TypeExpectation::Slicable => "indexable types (list, str)".to_string(),
            };

            let got = fmt_type(got.clone());

            return format!(" \n\
                ### expected \n\
                {exp}        \n\
                             \n\
                ### got      \n\
                {got}        \n\
            ")
        },
        oth => format!("{oth:?}")  // temp
    }
}