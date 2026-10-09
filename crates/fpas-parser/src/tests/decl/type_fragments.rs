//! Shared type-fragment grammar used by recovered editor chains.
//! Reference: `docs/pascal/tools/editor-integration.md`.

use super::*;
use crate::parse_type_expression;

#[test]
fn result_fragments_preserve_nested_success_and_error_types() {
    let (ty, diagnostics) =
        parse_type_expression("Result of Result of string, integer, Result of boolean, real");
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let TypeExpr::Result {
        ok_type, err_type, ..
    } = ty
    else {
        panic!("expected Result");
    };
    for (ty, success, error) in [
        (ok_type, "string", "integer"),
        (err_type, "boolean", "real"),
    ] {
        let TypeExpr::Result {
            ok_type, err_type, ..
        } = *ty
        else {
            panic!("expected nested Result");
        };
        assert_named(&ok_type, success);
        assert_named(&err_type, error);
    }
}

#[test]
fn type_fragments_preserve_mixed_containers_and_callable_parameters() {
    let source = "function(var Value: array of Result of string, integer): dict of string to Option of Result of string, boolean";
    let (ty, diagnostics) = parse_type_expression(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let TypeExpr::FunctionType {
        params,
        return_type,
        span,
    } = ty
    else {
        panic!("expected function type");
    };
    assert_eq!(span.text(source), Some(source));
    assert_eq!(params[0].mode, ParamMode::Var);
    assert!(
        matches!(&params[0].type_expr, TypeExpr::Array(inner, _) if matches!(**inner, TypeExpr::Result { .. }))
    );
    let TypeExpr::Dict {
        key_type,
        value_type,
        ..
    } = *return_type
    else {
        panic!("expected dictionary");
    };
    assert_named(&key_type, "string");
    assert!(
        matches!(*value_type, TypeExpr::Option { inner_type, .. } if matches!(*inner_type, TypeExpr::Result { .. }))
    );
}

#[test]
fn type_fragments_reject_missing_separators_and_trailing_tokens() {
    assert!(!parse_type_expression("").1.is_empty());
    for source in [
        "Result of string",
        "Result of string integer",
        "dict of string integer",
        "string, integer",
        "string;",
        "array integer",
    ] {
        let (_, diagnostics) = parse_type_expression(source);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.as_diagnostic().code == PARSE_EXPECTED_TOKEN),
            "{source}: {diagnostics:#?}"
        );
    }
}

#[test]
fn type_fragment_diagnostics_keep_lexer_errors_before_parser_errors() {
    let (_, diagnostics) = parse_type_expression("Result of string, @");
    assert!(matches!(
        diagnostics.first(),
        Some(ParseDiagnostic::Lexer(_))
    ));
    assert!(matches!(
        diagnostics.last(),
        Some(ParseDiagnostic::Parser(_))
    ));
}

#[test]
fn excessive_type_fragment_nesting_uses_the_shared_parser_limit() {
    use crate::parser::MAX_PARSER_NESTING_DEPTH;
    use fpas_diagnostics::codes::PARSE_NESTING_LIMIT_EXCEEDED;
    let source = format!("{}string", "array of ".repeat(MAX_PARSER_NESTING_DEPTH + 1));
    let (_, diagnostics) = parse_type_expression(&source);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].as_diagnostic().code,
        PARSE_NESTING_LIMIT_EXCEEDED
    );
}

fn assert_named(ty: &TypeExpr, expected: &str) {
    assert!(
        matches!(ty, TypeExpr::Named { id, .. } if id.parts == [expected]),
        "{ty:#?}"
    );
}
