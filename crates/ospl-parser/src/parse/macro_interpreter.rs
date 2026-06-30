use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ospl_common::ast::Position;

use crate::{lexer::token::{Span, Token}, parse::{PE, Parser, Res, macros::{MacroExpr, MacroIdent, MacroStmt, MacroTokType}}, tExp};

#[derive(Default, Clone, PartialEq)]
pub struct MacroScope {
    pub members: HashMap<MacroIdent, Rc<RefCell<MacroValue>>>
}

#[derive(Clone, PartialEq, Debug)]
pub enum MacroValue {
    /// A list of macro-time values
    List(Vec<Rc<RefCell<MacroValue>>>),
    Token(Span),
    Address(u64),
    Int(i64),
    Float(f64),
    Str(String),
}

pub struct MacroVM {
    stack: Vec<MacroScope>,
}

impl Default for MacroVM {
    fn default() -> Self {
        return Self {
            stack: vec![
                MacroScope::default()
            ]
        }
    }
}

impl MacroVM {
    pub fn top(&self) -> &MacroScope {
        return self.stack.last().expect("at least one macro scope is required");
    }

    pub fn top_mut(&mut self) -> &mut MacroScope {
        return self.stack.last_mut().expect("at least one macro scope is required");
    }

    pub fn eval<'b>(&mut self, p: &mut Parser<'b>, expr: &MacroExpr) -> Res<Rc<RefCell<MacroValue>>> {
        match expr {
            MacroExpr::Var(v) => {
                if let Some(memb) = self.top().members.get(v) {
                    return Ok(memb.clone());
                } else {
                    return Err(PE::NoSuchMacroVariable(v.0.to_owned()))
                }
            },
            MacroExpr::Address(a) => {
                return Ok(Rc::new(RefCell::new(MacroValue::Address(*a))));
            },
            MacroExpr::BinaryAdd(a, b) => {
                let aa = self.eval(p, &*a)?;
                let aaa = aa.borrow();

                let bb = self.eval(p, &*b)?;
                let bbb = bb.borrow();
                match (&*aaa, &*bbb) {
                    (MacroValue::Address(a), MacroValue::Address(b)) => {
                        return Ok(Rc::new(RefCell::new(MacroValue::Address(a + b))))
                    },
                    (MacroValue::List(a), b) => {
                        let mut a_copy = (*a).clone();
                        a_copy.push(Rc::new(RefCell::new(b.clone())));

                        assert_ne!(a_copy, *a);
                        return Ok(Rc::new(RefCell::new(MacroValue::List(a_copy))))
                    },
                    _ => return Err(PE::UnsupportedMacroOperation(aaa.to_owned(), bbb.to_owned()))
                }
            },
            MacroExpr::Do(things) => {
                let mut tokens = Vec::new();
                for thing in things {
                    tokens.extend(self.exec_stmt(p, thing)?);
                }

                let tokens = tokens.into_iter()
                    .map(|e| Rc::new(RefCell::new(MacroValue::Token(e))))
                    .collect();

                return Ok(Rc::new(RefCell::new(MacroValue::List(tokens))))
            },
            MacroExpr::ListGet(a, b) => {
                let aa = self.eval(p, &*a)?;
                let bb = self.eval(p, &*b)?;
                let bbb = bb.borrow();
                let aaa = aa.borrow();

                match (&*aaa, &*bbb) {
                    (MacroValue::List(l), MacroValue::Address(a)) => {
                        let Some(x) = l.get(*a as usize)
                        else { todo!("PUT AN ERROR HERE"); };

                        return Ok(x.clone());
                    },
                    _ => return Err(PE::UnsupportedMacroOperation(aaa.to_owned(), bbb.to_owned()))
                }
            },
            MacroExpr::LiteralToken(l) => {
                return Ok(Rc::new(RefCell::new(MacroValue::Token(l.to_owned()))));
            },
            MacroExpr::NewList(l) => {
                let mut new_l = Vec::new();
                for thing in l {
                    let eval = self.eval(p, thing)?;
                    new_l.push(eval);
                }

                return Ok(Rc::new(RefCell::new(MacroValue::List(new_l))));
            },
            MacroExpr::Next(n) => {
                let x = match n {
                    MacroTokType::Atom => p.parse_with_tokens(Parser::parse_atom)?.1,
                    MacroTokType::Expr => p.parse_with_tokens(Parser::parse_expr)?.1,
                    MacroTokType::Ident => p.parse_with_tokens(Parser::parse_ident)?.1,
                    MacroTokType::Stmt => p.parse_with_tokens(Parser::parse_stmt)?.1,
                    MacroTokType::Type => p.parse_with_tokens(Parser::parse_type)?.1,
                }.to_vec();

                let x = x
                    .into_iter()
                    .map(|e| Rc::new(RefCell::new(MacroValue::Token(e))))
                    .collect();

                return Ok(Rc::new(RefCell::new(MacroValue::List(x))));
            },
            MacroExpr::NextToken(_t) => todo!(),
            MacroExpr::Template(t) => {
                let mut items = Vec::new();
                for expr in t {
                    let eval = self.eval(p, expr)?;
                    match &*eval.borrow() {
                        MacroValue::List(t) => {
                            items.extend_from_slice(t);
                        },
                        MacroValue::Address(a) => items.push(
                            Rc::new(
                                RefCell::new(
                                    MacroValue::Token(
                                        Span::new(
                                            Position::default(),
                                            Token::AddressLiteral(*a)
                                        )
                                    )
                                )
                            )
                        ),
                        MacroValue::Int(a) => items.push(
                            Rc::new(
                                RefCell::new(
                                    MacroValue::Token(
                                        Span::new(
                                            Position::default(),
                                            Token::Integer(*a)
                                        )
                                    )
                                )
                            )
                        ),
                        MacroValue::Float(a) => items.push(
                            Rc::new(
                                RefCell::new(
                                    MacroValue::Token(
                                        Span::new(
                                            Position::default(),
                                            Token::Float(*a)
                                        )
                                    )
                                )
                            )
                        ),
                        MacroValue::Str(a) => items.push(
                            Rc::new(
                                RefCell::new(
                                    MacroValue::Token(
                                        Span::new(
                                            Position::default(),
                                            Token::StringLit(a.to_owned())
                                        )
                                    )
                                )
                            )
                        ),
                        MacroValue::Token(_) => items.push(eval.clone()),
                    }
                }

                return Ok(Rc::new(RefCell::new(MacroValue::List(items))));
            }
        }
    }

    pub fn exec_stmt<'b>(&mut self, p: &mut Parser<'b>, s: &MacroStmt) -> Res<Vec<Span>> {
        let mut token_stream = Vec::new();
        match s {
            MacroStmt::Define(id, expr) => {
                let v = self.eval(p, expr)?;
                self.top_mut().members.insert(id.to_owned(), v);
            },
            MacroStmt::Assign(id, expr) => {
                let x = self.eval(p, expr)?;
                if let Some(thing) = self.top_mut().members.get_mut(id) {
                    thing.replace(x.borrow().clone());
                }
            }
            MacroStmt::For(thing, thingys, do_these_things) => {
                let value = self.eval(p, thingys)?.clone();
                let MacroValue::List(ref t) = *value.borrow()
                else { todo!("put error here") };

                for v in t {
                    self.stack.push(self.top().clone());
                    self.top_mut().members.insert(thing.to_owned(), v.clone());

                    for thing in do_these_things {
                        token_stream.extend(self.exec_stmt(p, thing)?);
                    }

                    self.stack.pop();
                }
            },
            MacroStmt::Loop(macro_loop) => {
                p.expect(tExp!(LSquirly))?;
                loop {
                    self.stack.push(self.top().clone());
                    if *p.peek()?.token() == Token::RSquirly {
                        p.next()?;
                        break;
                    }

                    for stmt in macro_loop {
                        token_stream.extend(self.exec_stmt(p, stmt)?);
                    }
                    self.stack.pop();
                }
            },
            MacroStmt::Yeild(y) => {
                let x = self.eval(p, y)?;
                match &*x.borrow() {
                    MacroValue::List(l) => {
                        let toks: Vec<Span> = l
                            .iter()
                            .filter_map(|e| {
                                if let MacroValue::Token(t) = &*e.borrow() {
                                    return Some(t.to_owned())
                                } else { return None }
                            })
                            .collect();

                        token_stream.extend(toks);
                    },
                    _ => todo!("cannot do the thingy - put a real error here")
                }
            },
            MacroStmt::Debug(expr) => {
                let eval = self.eval(p, expr)?;
                println!("macro debug (call) for {:?}: {eval:?}", expr);
            }
        }

        return Ok(token_stream)
    }
}