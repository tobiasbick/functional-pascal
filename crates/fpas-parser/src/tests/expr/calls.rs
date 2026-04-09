use super::parse_expr;
use crate::ast::*;

#[test]
fn call_expr() {
    match parse_expr("Foo(1, 2)") {
        Expr::Call { args, .. } => assert_eq!(args.len(), 2),
        _ => panic!("expected Call"),
    }
}

#[test]
fn call_no_args_expr() {
    match parse_expr("Foo()") {
        Expr::Call { args, .. } => assert!(args.is_empty()),
        _ => panic!("expected Call"),
    }
}

#[test]
fn qualified_call_expr() {
    match parse_expr("Std.Math.Sqrt(4.0)") {
        Expr::Call {
            designator, args, ..
        } => {
            assert_eq!(designator.parts.len(), 3);
            assert_eq!(args.len(), 1);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn qualified_call_expr_std_unit_keyword_after_dot() {
    match parse_expr("Std.array.Length(x)") {
        Expr::Call {
            designator, args, ..
        } => {
            assert_eq!(designator.parts.len(), 3);
            assert_eq!(args.len(), 1);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn designator_expr_leading_std_unit_keyword() {
    match parse_expr("array.Length(x)") {
        Expr::Call {
            designator, args, ..
        } => {
            assert_eq!(designator.parts.len(), 2);
            assert_eq!(args.len(), 1);
        }
        _ => panic!("expected Call"),
    }
}
