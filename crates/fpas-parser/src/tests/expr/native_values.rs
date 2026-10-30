//! References and applications share one surface for names and reserved constructors.

use super::{Expr, parse_expr};

#[test]
fn native_callable_names_are_references_without_parentheses() {
    for name in [
        "array.Fill",
        "string.Chr",
        "Some",
        "Ok",
        "Error",
        "Math.Abs",
        "Text.Trim",
    ] {
        assert!(matches!(parse_expr(name), Expr::Designator(_)), "{name}");
    }
}

#[test]
fn native_callable_applications_keep_their_argument_nodes() {
    for source in ["array.Fill(Value := 7, Count := 3)", "string.Chr(N := 65)"] {
        assert!(matches!(parse_expr(source), Expr::Call { .. }), "{source}");
    }
    assert!(matches!(
        parse_expr("Some(Value := 1)"),
        Expr::OptionSome(_, _)
    ));
    assert!(matches!(parse_expr("Ok(Value := 1)"), Expr::ResultOk(_, _)));
    assert!(matches!(
        parse_expr("Error(Value := 'bad')"),
        Expr::ResultError(_, _)
    ));
}
