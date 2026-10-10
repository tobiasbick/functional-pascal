use super::parse_ok;
use crate::ast::*;

mod aggregates;
mod calls;
mod case_expressions;
mod closures;
mod designators;
mod if_expressions;
mod is_tests;
mod logical;
mod operators;
mod postfix;
mod precedence;
mod primitives;
mod retired_shifts;
mod standalone;

fn parse_expr(expr_src: &str) -> Expr {
    let src = format!("program T; begin return {expr_src}; end.");
    let program = parse_ok(&src);
    match &program.body[0] {
        Stmt::Return(Some(expr), _) => expr.clone(),
        _ => panic!("expected return with expression"),
    }
}
