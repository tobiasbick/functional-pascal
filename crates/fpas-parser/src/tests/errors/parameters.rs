//! Canonical formal-parameter diagnostics and recovery across all routine forms.

use super::*;
use fpas_diagnostics::codes::PARSE_INVALID_PARAMETER_SEPARATOR;

fn routine_sources(params: &str) -> Vec<String> {
    vec![
        format!("program P; function Add({params}): integer; begin return 1 end; begin end."),
        format!("program P; procedure Print({params}); begin end; begin end."),
        format!(
            "unit U; type R = record function Add({params}): integer; begin return 1 end; end;"
        ),
        format!("unit U; type R = record procedure Print({params}); begin end; end;"),
        format!("program P; begin Consume(function({params}): integer begin return 1 end) end."),
        format!("program P; begin Consume(procedure({params}) begin end) end."),
        format!("unit U; type F = function({params}): integer;"),
        format!("unit U; type F = procedure({params});"),
    ]
}

#[test]
fn comma_and_grouped_parameters_have_one_actionable_diagnostic() {
    for params in ["A: integer, B: integer", "A, B: integer"] {
        for source in routine_sources(params) {
            let (_, diagnostics) = parse_compilation_unit_with_errors(&source);
            let errors = diagnostics
                .iter()
                .filter_map(ParseDiagnostic::as_parser_error)
                .collect::<Vec<_>>();
            assert_eq!(errors.len(), 1, "{source}: {diagnostics:#?}");
            let error = errors[0];
            assert_eq!(error.code, PARSE_INVALID_PARAMETER_SEPARATOR, "{source}");
            assert_eq!(error.found.as_deref(), Some(","));
            assert!(
                error
                    .expected
                    .as_deref()
                    .unwrap()
                    .contains("individually typed")
            );
            assert!(
                error
                    .help
                    .as_deref()
                    .unwrap()
                    .contains("function Add(A: integer; B: integer): integer;")
            );
            let span = error.span.expect("comma range");
            assert_eq!(&source[span.offset()..span.end()], ",");
        }
    }
}

#[test]
fn canonical_parameters_and_commas_inside_types_remain_valid() {
    for params in [
        "A: integer; B: integer",
        "A: Result of integer, string; B: integer",
        "",
    ] {
        for source in routine_sources(params) {
            let (_, diagnostics) = parse_compilation_unit_with_errors(&source);
            assert!(diagnostics.is_empty(), "{source}: {diagnostics:#?}");
        }
    }
}

#[test]
fn recovery_keeps_the_body_following_declarations_and_call_arguments() {
    let source = "program P; procedure First(A, B: function(X: integer): integer); begin end; procedure Second(); begin end; begin Second() end.";
    let (unit, diagnostics) = parse_compilation_unit_with_errors(source);
    let errors = diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "{diagnostics:#?}");
    assert_eq!(errors[0].code, PARSE_INVALID_PARAMETER_SEPARATOR);
    let crate::CompilationUnit::Program(program) = unit else {
        panic!("program")
    };
    assert_eq!(program.declarations.len(), 2);
    assert_eq!(program.body.len(), 1);
    let (_, diagnostics) = parse_with_errors("program P; begin Add(1, 2) end.");
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn missing_closing_parenthesis_does_not_swallow_the_routine_body() {
    let (_, diagnostics) =
        parse_with_errors("program P; procedure Print(A, B: integer begin end; begin end.");
    assert!(
        diagnostics
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .any(|error| error.code == PARSE_INVALID_PARAMETER_SEPARATOR)
    );
    assert!(diagnostics.len() < 5, "recovery cascade: {diagnostics:#?}");
}
