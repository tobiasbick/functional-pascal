use super::parse_expr;
use crate::ast::*;

#[test]
fn empty_array() {
    match parse_expr("[]") {
        Expr::ArrayLiteral(elems, _) => assert!(elems.is_empty()),
        _ => panic!("expected ArrayLiteral"),
    }
}

#[test]
fn array_with_elements() {
    match parse_expr("[1, 2, 3]") {
        Expr::ArrayLiteral(elems, _) => assert_eq!(elems.len(), 3),
        _ => panic!("expected ArrayLiteral"),
    }
}

#[test]
fn record_literal() {
    match parse_expr("record X := 1; Y := 2; end") {
        Expr::RecordLiteral { fields, .. } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "X");
            assert_eq!(fields[1].name, "Y");
        }
        _ => panic!("expected RecordLiteral"),
    }
}
