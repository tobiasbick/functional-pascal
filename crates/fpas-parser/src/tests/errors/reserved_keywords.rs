//! Reserved block keywords must produce actionable identifier diagnostics.

use super::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_IDENTIFIER;

const RESERVED: &[&str] = &[
    "elsif", "when", "null", "ELSIF", "WHEN", "NULL", "ElSiF", "WhEn", "NuLl",
];

#[test]
fn reserved_block_keywords_are_rejected_in_declarations_with_rename_hints() {
    for keyword in RESERVED {
        for source in [
            format!("program {keyword}; begin end."),
            format!("unit {keyword};"),
            format!("program P; begin var {keyword}: integer := 1 end."),
            format!("program P; const {keyword}: integer := 1; begin end."),
            format!("program P; var {keyword}: integer := 1; begin end."),
            format!("program P; mutable var {keyword}: integer := 1; begin end."),
            format!("program P; type {keyword} = integer; begin end."),
            format!("program P; procedure F({keyword}: integer); begin end; begin end."),
            format!("program P; type R = record {keyword}: integer; end; begin end."),
            format!("program P; type E = enum {keyword}; end; begin end."),
        ] {
            let (_, diagnostics) = parse_compilation_unit_with_errors(&source);
            let error = diagnostics
                .iter()
                .filter_map(ParseDiagnostic::as_parser_error)
                .find(|error| error.code == PARSE_EXPECTED_IDENTIFIER)
                .unwrap_or_else(|| panic!("{source}: {diagnostics:#?}"));
            let hint = error.help.as_deref().unwrap_or_default();
            assert!(
                hint.contains("reserved") && hint.contains("Rename"),
                "{source}: {error:#?}"
            );
            let span = error
                .span
                .unwrap_or_else(|| panic!("missing keyword span: {error:#?}"));
            assert_eq!(&source[span.offset()..span.end()], *keyword, "{source}");
        }
    }
}

#[test]
fn qualified_reserved_names_have_the_same_rename_hint() {
    for keyword in RESERVED {
        for source in [
            format!("program P; begin Value.{keyword}() end."),
            format!("program P; uses App.{keyword}; begin end."),
            format!("program P; begin var Value: App.{keyword} := 1 end."),
            format!("program P; begin case Value of E.{keyword}: WriteLn('value') end end."),
        ] {
            let (_, diagnostics) = parse_with_errors(&source);
            assert!(
                diagnostics
                    .iter()
                    .filter_map(ParseDiagnostic::as_parser_error)
                    .any(|error| {
                        error.code == PARSE_EXPECTED_IDENTIFIER
                            && error
                                .help
                                .as_deref()
                                .is_some_and(|hint| hint.contains("Rename"))
                    }),
                "{source}: {diagnostics:#?}"
            );
        }
    }
}

#[test]
fn reserved_names_in_statements_and_expressions_have_rename_hints() {
    for keyword in RESERVED {
        for source in [
            format!("program P; begin {keyword} := 1 end."),
            format!("program P; begin Consume({keyword}) end."),
            format!("program P; begin var Values: array of integer := [{keyword}] end."),
        ] {
            let (_, diagnostics) = parse_with_errors(&source);
            assert!(
                diagnostics
                    .iter()
                    .filter_map(ParseDiagnostic::as_parser_error)
                    .any(|error| {
                        error.help.as_deref().is_some_and(|hint| {
                            hint.contains("reserved") && hint.contains("Rename")
                        })
                    }),
                "{source}: {diagnostics:#?}"
            );
        }
    }
}

#[test]
fn keyword_prefixes_strings_and_comments_remain_valid() {
    let source = "program P; begin var NullValue: integer := 1; var WhenValue: integer := 2; var ElsifValue: string := 'elsif when null'; // ELSIF WHEN NULL\nConsume(NullValue, WhenValue, ElsifValue) end.";
    let (_, diagnostics) = parse_with_errors(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn old_json_null_name_suggests_null_value() {
    let (_, diagnostics) = parse_with_errors("program P; begin Consume(JsonValue.Null) end.");
    assert!(
        diagnostics
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .any(|error| {
                error.code == PARSE_EXPECTED_IDENTIFIER
                    && error
                        .help
                        .as_deref()
                        .is_some_and(|hint| hint.contains("JsonValue.NullValue"))
            }),
        "{diagnostics:#?}"
    );
}

#[test]
fn recovery_retains_the_statement_after_a_reserved_name() {
    let (program, diagnostics) =
        parse_with_errors("program P; begin null := 1; WriteLn('after') end.");
    assert!(!diagnostics.is_empty());
    assert!(
        matches!(program.body.last(), Some(crate::Stmt::Call { .. })),
        "{:#?}",
        program.body
    );
}

#[test]
fn reserved_names_inside_declaration_lists_preserve_following_definitions() {
    for keyword in RESERVED {
        for source in [
            format!(
                "unit U; const First: integer := 1; {keyword}: integer := 2; Last: integer := 3;"
            ),
            format!(
                "unit U; var First: integer := 1; {keyword}: integer := 2; Last: integer := 3;"
            ),
            format!(
                "unit U; mutable var First: integer := 1; {keyword}: integer := 2; Last: integer := 3;"
            ),
            format!("unit U; type First = integer; {keyword} = integer; Last = integer;"),
        ] {
            let (unit, diagnostics) = parse_compilation_unit_with_errors(&source);
            let errors = diagnostics
                .iter()
                .filter_map(ParseDiagnostic::as_parser_error)
                .collect::<Vec<_>>();
            assert_eq!(errors.len(), 1, "{source}: {diagnostics:#?}");
            assert_eq!(errors[0].code, PARSE_EXPECTED_IDENTIFIER);
            assert!(
                errors[0]
                    .help
                    .as_deref()
                    .is_some_and(|hint| hint.contains("Rename"))
            );
            let crate::CompilationUnit::Unit(unit) = unit else {
                panic!("expected a unit")
            };
            assert_eq!(
                unit.declarations.len(),
                3,
                "{source}: {:#?}",
                unit.declarations
            );
        }
    }
}
