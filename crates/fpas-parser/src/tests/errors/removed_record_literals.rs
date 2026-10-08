//! Rejection of the removed `record ... end` literal.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::parse_with_errors;
use crate::ParseDiagnostic;
use fpas_diagnostics::codes::PARSE_REMOVED_RECORD_LITERAL;

fn removed_literal_hints(source: &str) -> Vec<String> {
    let (_, diagnostics) = parse_with_errors(source);
    diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .filter(|error| error.code == PARSE_REMOVED_RECORD_LITERAL)
        .map(|error| error.help.clone().unwrap_or_default())
        .collect()
}

#[test]
fn literal_with_declared_type_names_the_constructor() {
    for source in [
        "program T; const P: Point := record X := 1; Y := 2; end; begin end.",
        "program T; begin var P: Point := record X := 1; Y := 2; end; end.",
        "program T; type R = record P: Point := record X := 1; Y := 2; end; end record; begin end.",
    ] {
        let hints = removed_literal_hints(source);
        assert_eq!(hints.len(), 1, "{source}: {hints:#?}");
        assert!(
            hints[0].contains("`Point(X := ..., Y := ...)`"),
            "{source}: {hints:#?}"
        );
    }
}

#[test]
fn literal_with_qualified_declared_type_names_the_qualified_constructor() {
    let hints =
        removed_literal_hints("program T; const P: Geo.Point := record X := 1; end; begin end.");
    assert_eq!(hints.len(), 1, "{hints:#?}");
    assert!(hints[0].contains("`Geo.Point(X := ...)`"), "{hints:#?}");
}

#[test]
fn literal_without_declared_type_uses_a_placeholder_type_name() {
    for source in [
        "program T; begin return record X := 1; end; end.",
        "program T; begin Draw(record X := 1; end); end.",
        "program T; const P: Point := Wrap(record X := 1; end); begin end.",
        "program T; const Points: array of Point := [record X := 1; end]; begin end.",
        "program T; begin P := record X := 1; end; end.",
    ] {
        let hints = removed_literal_hints(source);
        assert_eq!(hints.len(), 1, "{source}: {hints:#?}");
        assert!(
            hints[0].contains("`TypeName(X := ...)`"),
            "{source}: {hints:#?}"
        );
    }
}

#[test]
fn empty_literal_shows_a_field_placeholder() {
    let hints = removed_literal_hints("program T; const P: Point := record end; begin end.");
    assert_eq!(hints.len(), 1, "{hints:#?}");
    assert!(hints[0].contains("`Point(Field := Value)`"), "{hints:#?}");
}

#[test]
fn rejected_literal_keeps_the_following_statement() {
    let (program, errors) =
        parse_with_errors("program T; begin X := record Field := 1; end; Y := 1; end.");

    assert!(
        errors.iter().any(|error| error
            .as_parser_error()
            .is_some_and(|error| error.code == PARSE_REMOVED_RECORD_LITERAL)),
        "{errors:#?}"
    );
    assert_eq!(program.body.len(), 2, "program AST: {program:#?}");
    assert!(matches!(program.body[1], crate::Stmt::Assign { .. }));
}
