use super::parse_expr;
use crate::ast::*;

#[test]
fn explicit_var_argument_retains_qualified_storage_and_index_expressions() {
    let Expr::Call { args, .. } = parse_expr("Change(var Data.Items[Select()], 1)") else {
        panic!("call");
    };
    let Expr::VarArgument(target, span) = &args[0] else {
        panic!("var storage");
    };
    assert_eq!(target.parts.len(), 3);
    assert!(matches!(
        &target.parts[2],
        DesignatorPart::Index(Expr::Call { .. }, _)
    ));
    assert!(span.offset < target.span.offset);
    assert_eq!(span.end_offset(), target.span.end_offset());
}

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
    match parse_expr("Std.Arrays.Length(x)") {
        Expr::Call {
            designator, args, ..
        } => {
            assert_eq!(designator.parts.len(), 3);
            assert_eq!(args.len(), 1);
        }
        _ => panic!("expected Call"),
    }
}
