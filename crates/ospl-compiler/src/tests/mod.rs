#![allow(unused)]  // required for RA
use std::collections::HashMap;
use crate::{Compiler, ast::{Expr, Expression, FunctionType, FunctionValue, LV, LValue, Statement, Stmt, Scope, Type}};

#[test]
fn retscope() {
    let mut c = Compiler::new();
    let mut ob = Vec::new();

    let mut output_type = Scope::default();
    output_type.declare("x".to_string(), 0, ospl_common::ast::Type::Int);

    c.compile_stmt(&Statement::test(
        Stmt::Define(
            "F".to_string(),
            Expression::test(Expr::Literal(
                crate::ast::Literal::Function(FunctionValue {
                    args: Vec::new(),
                    block: vec![
                        Statement::test(Stmt::Define(
                            "x".to_string(),
                            Expression::test(
                                Expr::Literal(crate::ast::Literal::Int(0))
                            )
                        )),
                        Statement::test(Stmt::ReturnScope)
                    ],
                    ftype: FunctionType {
                        args: Vec::new(),
                        // captures: Vec::new(),
                        ret: Type::Scope(output_type)
                    }
                }))
            )
        )
    ), &mut ob);

    c.compile_stmt(&Statement::test(
        Stmt::Define("s".to_string(), Expression::test(
            Expr::Call(
                Expression::test(
                    Expr::LValue(
                        crate::ast::LValue::test(
                            LV::Variable("F".to_string())
                        )
                    )
                ),
                Vec::new()
            )
        )
    )), &mut ob);

    c.compile_stmt(&Statement::test(
        Stmt::Define("x".to_string(), Expression::test(
            Expr::LValue(LValue::test(
                LV::Property(
                    LValue::test(LV::Variable("s".to_string())),
                    "x".to_string()
                )
            ))
        ))
    ), &mut ob);

    let (_, ty) = c.stack.top().get_combined("x").expect("x didn't get defined");
    assert_eq!(*ty, Type::Int);
}