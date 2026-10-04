use super::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

#[test]
fn function_type_expr() {
    let p = parse_ok(
        r#"program T;  const F: function(X: integer): integer := Add; begin null; end program;"#,
    );
    match &p.declarations[0] {
        Decl::Const(v) => {
            assert!(matches!(
                v.type_expr.as_ref().unwrap(),
                TypeExpr::FunctionType { .. }
            ));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn procedure_type_expr() {
    let p = parse_ok(
        r#"program T;  const P: procedure(X: integer) := DoStuff; begin null; end program;"#,
    );
    match &p.declarations[0] {
        Decl::Const(v) => {
            assert!(matches!(
                v.type_expr.as_ref().unwrap(),
                TypeExpr::ProcedureType { .. }
            ));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn built_in_and_callable_type_forms_parse() {
    for type_expr in [
        "array of (integer)",
        "channel of (string)",
        "option of (array of (integer))",
        "result of (integer, string)",
        "dict of (string, array of (integer))",
        "function(X: integer): integer",
        "procedure(X: integer)",
    ] {
        let source =
            format!(r#"program T; const Value: {type_expr} := nil; begin null; end program;"#);
        let (_, errors) = parse_with_errors(&source);
        assert!(errors.is_empty(), "{type_expr}: {errors:#?}");
    }
}

#[test]
fn built_in_and_callable_type_forms_require_their_separators() {
    for type_expr in [
        "array integer",
        "channel string",
        "option integer",
        "result integer, string",
        "result of integer string",
        "dict of string integer",
        "function(X: integer) integer",
        "procedure(X: integer",
    ] {
        let source =
            format!(r#"program T; const Value: {type_expr} := nil; begin null; end program;"#);
        let (_, errors) = parse_with_errors(&source);
        assert!(
            errors.iter().any(|error| error
                .as_parser_error()
                .is_some_and(|error| error.code == PARSE_EXPECTED_TOKEN)),
            "accepted malformed type `{type_expr}`: {errors:#?}"
        );
    }
}
