#![allow(unused)]  // required for RA
use std::collections::HashMap;
use crate::{Compiler, ast::{Expr, Expression, FunctionType, FunctionValue, LV, LValue, Statement, Stmt}};

#[test]
fn retscope() {
    let mut c = Compiler::new();
    let mut ob = Vec::new();

    let mut output_type_map = HashMap::new();
    output_type_map.insert("x".to_string(), crate::Store {
        ty: crate::Type::Int,
        var: 0
    });

    c.compile_stmt(&Statement {
        inner: Box::new(Stmt::Define(
            "F".to_string(),
            Expression {inner: Box::new(Expr::Literal(
                crate::ast::Literal::Function(FunctionValue {
                    block: vec![
                        Statement { inner: Box::new(Stmt::Define(
                            "x".to_string(),
                            Expression { inner: Box::new(
                                Expr::Literal(crate::ast::Literal::Int(0))
                            ) }
                        ))},
                        Statement { inner: Box::new(Stmt::ReturnScope) }
                    ],
                    ftype: FunctionType {
                        args: Vec::new(),
                        captures: Vec::new(),
                        ret: crate::Type::Scope(crate::Scope {
                            map: output_type_map,
                            next_id: 1
                        })
                    }
                }))
            )}
        ))
    }, &mut ob);

    c.compile_stmt(&Statement { inner: Box::new(
        Stmt::Define("s".to_string(), Expression { inner: Box::new(
            Expr::Call(
                Expression { inner: Box::new(
                    Expr::LValue(
                        crate::ast::LValue { inner: Box::new(
                            LV::Variable("F".to_string())
                        ) }
                    )
                )},
                Vec::new()
            )
        ) })
    ) }, &mut ob);

    c.compile_stmt(&Statement::test(
        Stmt::Define("x".to_string(), Expression::test(
            Expr::LValue(LValue::test(
                LV::Property(
                    LValue { inner: Box::new(LV::Variable("s".to_string())) },
                    "x".to_string()
                )
            ))
        ))
    ), &mut ob);

    let x = c.stack.top().map.get("x").expect("x didn't get defined");
    assert_eq!(x.ty, crate::Type::Int);
}