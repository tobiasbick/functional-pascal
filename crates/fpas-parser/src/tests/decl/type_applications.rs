//! Canonical built-in type applications and recovery from rejected forms.
//! See `docs/pascal/language/types/generics.md`.

use super::*;
use crate::parse_type_expression;

#[test]
fn nested_result_arguments_keep_their_spans_and_nominal_names() {
    let source =
        "Result of (\n  Option of Geometry.Point,\n  Result of (array of integer, string)\n)";
    let (ty, diagnostics) = parse_type_expression(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let TypeExpr::Result {
        ok_type,
        err_type,
        span,
    } = ty
    else {
        panic!("expected Result");
    };
    assert_eq!(span.text(source), Some(source));
    let TypeExpr::Option { inner_type, span } = *ok_type else {
        panic!("expected Option");
    };
    assert_eq!(span.text(source), Some("Option of Geometry.Point"));
    assert!(matches!(*inner_type, TypeExpr::Named { id, .. } if id.parts == ["Geometry", "Point"]));
    assert!(
        matches!(*err_type, TypeExpr::Result { ok_type, err_type, .. }
        if matches!(*ok_type, TypeExpr::Array(_, _))
        && matches!(&*err_type, TypeExpr::Named { id, .. } if id.parts == ["string"]))
    );
}

#[test]
fn result_requires_exactly_two_arguments_and_both_parentheses() {
    for source in [
        "Result of ()",
        "Result of (integer)",
        "Result of (integer,)",
        "Result of (, string)",
        "Result of (integer, string, boolean)",
        "Result of (integer string)",
        "Result of (integer, string",
        "Option of (integer)",
    ] {
        let (_, diagnostics) = parse_type_expression(source);
        assert!(
            diagnostics.iter().any(|diagnostic| matches!(
                diagnostic.as_diagnostic().code,
                PARSE_EXPECTED_TOKEN | fpas_diagnostics::codes::PARSE_EXPECTED_IDENTIFIER
            )),
            "accepted malformed application {source}: {diagnostics:#?}"
        );
    }
}

#[test]
fn unparenthesized_result_is_rejected_without_losing_the_next_declaration() {
    let source = "program T; type Old = Result of integer, string; type Next = Option of integer; begin end.";
    let (program, diagnostics) = parse_with_errors(source);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let error = diagnostics[0].as_diagnostic();
    assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("Result of (integer, string)")),
        "{error:?}"
    );
    assert_eq!(program.declarations.len(), 2);
    assert!(
        matches!(&program.declarations[1], Decl::TypeDef(declaration) if declaration.name == "Next")
    );
}

#[test]
fn angle_bracket_applications_show_the_of_form_and_recover() {
    for (application, replacement) in [
        ("Result<integer, string>", "Result of (T, E)"),
        ("Option<integer>", "Option of T"),
        ("array<integer>", "array of T"),
        ("channel<integer>", "channel of T"),
        ("task<integer>", "task of T"),
        ("dict<string, integer>", "dict of K to V"),
        ("Geometry.Lookup<string>", "Geometry.Lookup of T"),
        ("Pair<integer, string>", "Pair of (T1, T2)"),
    ] {
        let source =
            format!("program T; type Bad = {application}; type Next = integer; begin end.");
        let (program, diagnostics) = parse_with_errors(&source);
        assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:#?}");
        let error = diagnostics[0].as_diagnostic();
        assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
        assert!(
            error
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains(replacement)),
            "{error:?}"
        );
        let span = error.span.expect("application span");
        assert_eq!(&source[span.offset()..span.offset() + span.length()], "<");
        assert!(
            matches!(&program.declarations[1], Decl::TypeDef(declaration) if declaration.name == "Next")
        );
    }
}

#[test]
fn rejected_nested_angle_arguments_report_each_application() {
    let (_, diagnostics) = parse_type_expression("Result<Result<integer, string>, Option<string>>");
    assert_eq!(diagnostics.len(), 3, "{diagnostics:#?}");
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic
            .as_diagnostic()
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains(" of "))
    }));
}
