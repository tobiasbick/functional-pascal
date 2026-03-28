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
