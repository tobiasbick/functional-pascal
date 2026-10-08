//! `Value is Pattern` parsing at comparison precedence.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`

use super::*;
use crate::tests::parse_with_errors;
use crate::{ParseDiagnostic, Pattern};

#[test]
fn is_test_binds_tighter_than_and() {
    let expr = parse_expr("X is Some(const V) and V > 0");
    let Expr::BinaryOp {
        op: BinaryOp::And,
        left,
        ..
    } = expr
    else {
        panic!("expected `and` at the top: {expr:#?}");
    };
    let Expr::Is { value, pattern, .. } = *left else {
        panic!("expected `is` on the left");
    };
    assert!(matches!(*value, Expr::Designator(_)));
    assert!(matches!(*pattern, Pattern::Destructure { .. }));
}

#[test]
fn is_value_patterns_stop_before_logical_operators() {
    let expr = parse_expr("X is 0 and Ready");
    let Expr::BinaryOp {
        op: BinaryOp::And,
        left,
        ..
    } = expr
    else {
        panic!("expected `and` at the top: {expr:#?}");
    };
    assert!(matches!(
        *left,
        Expr::Is { ref pattern, .. } if matches!(**pattern, Pattern::Value(Expr::Integer(0, _)))
    ));
}

#[test]
fn is_test_accepts_nested_and_qualified_variant_patterns() {
    let expr = parse_expr("Msg is TuiMsg.Resize(const W, _)");
    let Expr::Is { pattern, .. } = expr else {
        panic!("expected `is`");
    };
    assert!(matches!(*pattern, Pattern::Variant { ref fields, .. } if fields.len() == 2));
}

#[test]
fn is_is_reserved() {
    let (_, errors) = parse_with_errors("program T; var Is: integer := 1; begin end.");
    assert!(
        errors.iter().any(|error| matches!(error, ParseDiagnostic::Parser(diagnostic)
            if diagnostic.help.as_deref().is_some_and(|help| help.contains("`is` is a reserved keyword")))),
        "{errors:#?}"
    );
}
