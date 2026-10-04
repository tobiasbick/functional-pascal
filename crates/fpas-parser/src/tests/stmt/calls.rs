use super::super::parse_with_errors;
use super::body_stmts;
use crate::{ParseDiagnostic, ast::*};
use fpas_diagnostics::codes::PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM;

#[test]
fn call_no_args() {
    let stmts = body_stmts(r#"program T; begin Foo(); end program;"#);
    match &stmts[0] {
        Stmt::Call { args, .. } => {
            assert!(args.is_empty());
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn call_with_args() {
    let stmts = body_stmts(r#"program T; begin WriteLn('hello', 42); end program;"#);
    match &stmts[0] {
        Stmt::Call { args, .. } => {
            assert_eq!(args.len(), 2);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn qualified_call() {
    let stmts = body_stmts(r#"program T; begin Std.Console.WriteLn('hello'); end program;"#);
    match &stmts[0] {
        Stmt::Call { designator, .. } => {
            assert_eq!(designator.parts.len(), 3);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn bare_zero_argument_call_requires_parentheses() {
    let (_, errors) = parse_with_errors("program T; begin Foo end.");
    let diagnostic = errors.iter().find_map(|error| match error {
        ParseDiagnostic::Parser(diagnostic)
            if diagnostic.code == PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM =>
        {
            Some(diagnostic)
        }
        _ => None,
    });

    let diagnostic = diagnostic.unwrap_or_else(|| {
        panic!("expected invalid call-or-assignment diagnostic, got: {errors:#?}")
    });
    assert_eq!(
        diagnostic.help.as_deref(),
        Some("Add `()` to call a zero-argument function or procedure.")
    );
}

#[test]
fn bare_qualified_zero_argument_call_requires_parentheses() {
    let (_, errors) = parse_with_errors("program T; begin Std.Console.Clear end.");

    assert!(
        errors.iter().any(|error| matches!(
            error,
            ParseDiagnostic::Parser(diagnostic)
                if diagnostic.code == PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM
        )),
        "expected invalid call-or-assignment diagnostic, got: {errors:#?}"
    );
}

#[test]
fn postfix_callable_field_chain_is_expression_statement() {
    let stmts = body_stmts(r#"program T; begin Factory.Create().Destroy(); end program;"#);
    match &stmts[0] {
        Stmt::Expression {
            expr: Expr::Postfix { operations, .. },
            ..
        } => {
            assert_eq!(operations.len(), 2);
            assert!(
                matches!(&operations[1], PostfixOperation::Call { args, .. } if args.is_empty())
            );
            assert!(matches!(
                &operations[0],
                PostfixOperation::Field { name, .. } if name == "Destroy"
            ));
        }
        other => panic!("expected postfix expression statement, got {other:?}"),
    }
}

#[test]
fn parenthesized_postfix_callable_field_chain_is_expression_statement() {
    let stmts = body_stmts(r#"program T; begin (Factory.Create()).Destroy(); end program;"#);
    assert!(matches!(
        &stmts[0],
        Stmt::Expression {
            expr: Expr::Postfix { .. },
            ..
        }
    ));
}
