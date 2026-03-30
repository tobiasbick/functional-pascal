use super::parse_expr;
use crate::ast::*;

#[test]
fn mul_before_add() {
    match parse_expr("1 + 2 * 3") {
        Expr::BinaryOp {
            op: BinaryOp::Add,
            right,
            ..
        } => {
            assert!(matches!(
                *right,
                Expr::BinaryOp {
                    op: BinaryOp::Mul,
                    ..
                }
            ));
        }
        _ => panic!("expected Add at top"),
    }
}

#[test]
fn comparison_lowest_precedence() {
    match parse_expr("A + B = C") {
        Expr::BinaryOp {
            op: BinaryOp::Eq,
            left,
            ..
        } => {
            assert!(matches!(
                *left,
                Expr::BinaryOp {
                    op: BinaryOp::Add,
                    ..
                }
            ));
        }
        _ => panic!("expected Eq at top"),
    }
}

#[test]
fn parens_override_precedence() {
    match parse_expr("(1 + 2) * 3") {
        Expr::BinaryOp {
            op: BinaryOp::Mul,
            left,
            ..
        } => {
            assert!(matches!(*left, Expr::Paren(_, _)));
        }
        _ => panic!("expected Mul at top"),
    }
}

#[test]
fn not_highest_precedence() {
    match parse_expr("not A and B") {
        Expr::BinaryOp {
            op: BinaryOp::And,
            left,
            ..
        } => {
            assert!(matches!(
                *left,
                Expr::UnaryOp {
                    op: UnaryOp::Not,
                    ..
                }
            ));
        }
        _ => panic!("expected And at top"),
    }
}

#[test]
fn left_associative_add() {
    match parse_expr("1 - 2 + 3") {
        Expr::BinaryOp {
            op: BinaryOp::Add,
            left,
            ..
        } => {
            assert!(matches!(
                *left,
                Expr::BinaryOp {
                    op: BinaryOp::Sub,
                    ..
                }
            ));
        }
        _ => panic!("expected Add at top, Sub on left"),
    }
}

#[test]
fn try_has_unary_precedence() {
    match parse_expr("try GetVal() * 3") {
        Expr::BinaryOp {
            op: BinaryOp::Mul,
            left,
            ..
        } => {
            assert!(matches!(*left, Expr::Try(_, _)));
        }
        _ => panic!("expected Mul at top, Try on left"),
    }
}
