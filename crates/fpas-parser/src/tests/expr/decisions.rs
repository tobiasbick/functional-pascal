//! Required fallbacks and expression-only branches preserve named closer ownership.

use super::parse_expr;
use crate::{Expr, Stmt};

#[test]
fn decisions_are_values_inside_returns_arguments_and_postfix_chains() {
    assert!(matches!(
        parse_expr("if true then 42 elsif false then 1 else 0 end if"),
        Expr::If(_)
    ));
    assert!(matches!(
        parse_expr("case true of when true: 42; when false: 0; end case"),
        Expr::Case(_)
    ));
    assert!(matches!(
        parse_expr("(if true then First else Second end if)()"),
        Expr::Postfix { .. }
    ));
    let (program, errors) =
        crate::parse("program Main; begin if true then null; else null; end if; end program;");
    assert!(errors.is_empty());
    assert!(matches!(program.body[0], Stmt::If { .. }));
}

#[test]
fn decision_values_reject_missing_else_statement_bodies_and_extra_branch_values() {
    for expression in [
        "if true then 1 end if",
        "if true then begin null; end; else 0 end if",
        "if true then 1; else 0 end if",
        "case true of when true: 1; 2; when false: 0; end case",
        "case true of else 1; end case",
    ] {
        let (_, errors) = crate::parse(&format!(
            "program Main; begin return {expression}; end program;"
        ));
        assert!(!errors.is_empty(), "accepted {expression}");
    }
}
