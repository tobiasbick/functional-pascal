use super::parse_expr;
use crate::ast::*;

#[test]
fn add() {
    match parse_expr("1 + 2") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Add),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn subtract() {
    match parse_expr("5 - 3") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Sub),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn multiply() {
    match parse_expr("2 * 3") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Mul),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn divide() {
    match parse_expr("10 / 3") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::RealDiv),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn int_div() {
    match parse_expr("10 div 3") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::IntDiv),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn modulo() {
    match parse_expr("10 mod 3") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Mod),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn and_op() {
    match parse_expr("true and false") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::And),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn or_op() {
    match parse_expr("true or false") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Or),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn xor_op() {
    match parse_expr("true xor false") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Xor),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn shl_op() {
    match parse_expr("1 shl 4") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Shl),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn shr_op() {
    match parse_expr("16 shr 4") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Shr),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn equal() {
    match parse_expr("1 = 1") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Eq),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn not_equal() {
    match parse_expr("1 <> 2") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::NotEq),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn less_than() {
    match parse_expr("1 < 2") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Lt),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn greater_than() {
    match parse_expr("2 > 1") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Gt),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn less_equal() {
    match parse_expr("1 <= 2") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::LtEq),
        _ => panic!("expected BinaryOp"),
    }
}

#[test]
fn greater_equal() {
    match parse_expr("2 >= 1") {
        Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::GtEq),
        _ => panic!("expected BinaryOp"),
    }
}
