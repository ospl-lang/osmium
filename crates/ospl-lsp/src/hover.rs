use std::fmt::Write;
use ospl_compiler::ast::{Expr, FunctionValue, Literal, Position, Statement, Stmt};

pub fn gen_fn_hover(f: &FunctionValue) -> String {
    let mut out = String::new();

    let _ = writeln!(&mut out, "fn(");
    for (name, arg) in f.args.iter().zip(f.ftype.args.iter()) {
        let name = &name.name;
        let _ = writeln!(&mut out, "\t{name}: {}", arg);
    }
    let _ = writeln!(&mut out, ") -> {}", f.ftype.ret);

    return out
}

pub fn gen_hover(s: &Statement) -> Option<(Position, String)> {
    match &*s.inner {
        Stmt::Define(d) => {
            // let col = s.at.column + d.name.len();
            // let end = Position { column: col, ..s.at };
            
            let end = d.rhs.at.clone();
            let mut lines = s.notes.lines();
            let Some(doc_line) = lines.next()
            else { return None; };

            let mut out_str = String::new();
            // let _ = writeln!(&mut out_str, "`{}:{}:{}`\n***\n", s.file, s.at.line, s.at.column);

            let doc_line = doc_line.trim();
            match doc_line {
                "doc" => {
                    let x = match &*d.rhs.inner {
                        Expr::Literal(l) => {
                            match l {
                                Literal::Function(f) => format!("```ospl\n{}\n```", gen_fn_hover(&f)),
                                a => format!("{a:?}")
                            }
                        },
                        _ => "".to_string()
                    };

                    out_str.push_str(&x);

                    out_str.push_str("\n***\n");
                    out_str.extend(lines.map(|l| format!("{l}\n")));
                },
                _ => {
                    out_str.push_str("Unknown, notes are as follows:\n");
                    out_str.push_str("```\n");
                    out_str.extend(lines.map(|l| format!("{l}\n")));
                    out_str.push_str("```\n");
                }
            }

            return Some((end, out_str))
        },
        _ => None
    }
}