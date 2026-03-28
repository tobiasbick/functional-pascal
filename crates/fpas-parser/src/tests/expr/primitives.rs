use super::parse_expr;
use crate::ast::*;

#[test]
fn integer_literal() {
    assert!(matches!(parse_expr("42"), Expr::Integer(42, _)));
}

#[test]
fn real_literal() {
    match parse_expr("3.14") {
        Expr::Real(v, _) => assert!((v - 3.14).abs() < 1e-10),
        _ => panic!("expected Real"),
    }
}

#[test]
fn string_literal() {
    assert!(matches!(parse_expr("'hello'"), Expr::Str(s, _) if s == "hello"));
}

#[test]
fn bool_true() {
    assert!(matches!(parse_expr("true"), Expr::Bool(true, _)));
}

#[test]
fn bool_false() {
    assert!(matches!(parse_expr("false"), Expr::Bool(false, _)));
}

#[test]
fn unary_negate() {
    match parse_expr("-42") {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(op, UnaryOp::Negate);
            assert!(matches!(*operand, Expr::Integer(42, _)));
        }
        _ => panic!("expected UnaryOp"),
    }
}

#[test]
fn unary_not() {
    match parse_expr("not true") {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(op, UnaryOp::Not);
            assert!(matches!(*operand, Expr::Bool(true, _)));
        }
        _ => panic!("expected UnaryOp"),
    }
}

#[test]
fn double_negate() {
    match parse_expr("- -1") {
        Expr::UnaryOp {
            op: UnaryOp::Negate,
            operand,
            ..
        } => {
            assert!(matches!(
                *operand,
                Expr::UnaryOp {
                    op: UnaryOp::Negate,
                    ..
                }
            ));
        }
        _ => panic!("expected nested UnaryOp"),
    }
}
