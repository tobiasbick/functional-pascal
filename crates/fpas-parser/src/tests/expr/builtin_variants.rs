//! Builtin constructors have explicit owners and retain their wrapper AST kinds.

use super::parse_expr;
use crate::Expr;

#[test]
fn qualified_builtin_variants_parse_nested_payloads_and_complete_spans() {
    assert!(matches!(parse_expr("Option.None"), Expr::OptionNone(_)));
    assert!(matches!(
        parse_expr("Option.Some(Result.Ok(42))"),
        Expr::OptionSome(..)
    ));
    assert!(matches!(
        parse_expr("Result.Error('failed')"),
        Expr::ResultError(..)
    ));
    let expression = parse_expr("rEsUlT.oK(oPtIoN.sOmE(42))");
    assert!(matches!(expression, Expr::ResultOk(..)));
    assert_eq!(expression.span().length, "rEsUlT.oK(oPtIoN.sOmE(42))".len());
}

#[test]
fn builtin_variants_reject_foreign_members_missing_qualifiers_and_wrong_payload_arity() {
    for value in [
        "Option.Ok(42)",
        "Result.Some(42)",
        "Option Some(42)",
        "Result.None",
        "Result.Ok()",
        "Option.Some(1, 2)",
    ] {
        let (_, errors) =
            crate::parse(&format!("program Main; begin return {value}; end program;"));
        assert!(!errors.is_empty(), "accepted {value}");
    }
}

#[test]
fn bare_builtin_variants_are_rejected_with_qualified_hints_in_values_and_patterns() {
    for (value, owner) in [
        ("Some(1)", "Option"),
        ("None", "Option"),
        ("Ok(1)", "Result"),
        ("Error('failed')", "Result"),
    ] {
        for source in [
            format!("program T; begin discard {value}; end program;"),
            format!("program T; begin case Value of when {value}: null; end case; end program;"),
        ] {
            let (_, errors) = crate::parse(&source);
            assert!(
                errors
                    .iter()
                    .any(
                        |error| error.as_parser_error().is_some_and(|error| error.message
                            == "Builtin variant requires its type qualifier"
                            && error
                                .help
                                .as_deref()
                                .is_some_and(|hint| hint.contains(owner)))
                    ),
                "{source}: {errors:#?}"
            );
        }
    }
}

#[test]
fn bare_builtin_pattern_payloads_still_parse_without_cascades() {
    let (_, errors) = crate::parse(
        "program T; begin case Value of when Some(const V) if V > 0: null; when None: null; when Error(_): null; end case; end program;",
    );
    assert_eq!(errors.len(), 3, "{errors:#?}");
    assert!(errors.iter().all(|error| {
        error
            .as_parser_error()
            .is_some_and(|error| error.message == "Builtin variant requires its type qualifier")
    }));
}
