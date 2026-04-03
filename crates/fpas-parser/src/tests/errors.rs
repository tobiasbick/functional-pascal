use super::{parse_compilation_unit_with_errors, parse_with_errors};
use crate::ParseDiagnostic;

#[test]
fn missing_program_keyword() {
    let (_, errs) = parse_with_errors("Hello; begin end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_semicolon_after_program() {
    let (_, errs) = parse_with_errors("program Hello begin end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_begin() {
    let (_, errs) = parse_with_errors("program T; end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_end() {
    let (_, errs) = parse_with_errors("program T; begin .");
    assert!(!errs.is_empty());
}

#[test]
fn missing_closing_paren() {
    let (_, errs) = parse_with_errors("program T; begin Foo(1, 2 end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_then() {
    let (_, errs) = parse_with_errors("program T; begin if X > 0 Y := 1 end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_do_in_for() {
    let (_, errs) = parse_with_errors("program T; begin for I: integer := 0 to 9 X := I end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_do_in_while() {
    let (_, errs) = parse_with_errors("program T; begin while true X := 1 end.");
    assert!(!errs.is_empty());
}

#[test]
fn while_missing_condition() {
    let (_, errs) = parse_with_errors("program T; begin while do X := 1 end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_until() {
    let (_, errs) = parse_with_errors("program T; begin repeat X := 1  X = 10 end.");
    assert!(!errs.is_empty());
}

#[test]
fn repeat_missing_condition() {
    let (_, errs) = parse_with_errors("program T; begin repeat X := 1 until end.");
    assert!(!errs.is_empty());
}

#[test]
fn repeat_empty_body_missing_until() {
    let (_, errs) = parse_with_errors("program T; begin repeat end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_colon_assign() {
    let (_, errs) = parse_with_errors("program T; begin var X: integer 42 end.");
    assert!(!errs.is_empty());
}

#[test]
fn errors_have_hints() {
    let (_, errs) = parse_with_errors("program T; begin @ end.");
    for err in &errs {
        match err {
            ParseDiagnostic::Lexer(diagnostic) => {
                assert!(
                    diagnostic
                        .help
                        .as_deref()
                        .map(|hint| !hint.is_empty())
                        .unwrap_or(false),
                    "Error missing hint: {}",
                    diagnostic.message
                );
            }
            ParseDiagnostic::Parser(error) => assert!(
                error.help.as_deref().is_some_and(|hint| !hint.is_empty()),
                "Error missing hint: {}",
                error.message
            ),
        }
    }
}

#[test]
fn error_recovery_continues() {
    // Even with an error, parsing should still make progress
    let (prog, errs) = parse_with_errors("program T; begin X := 1; Y := end.");
    assert!(!errs.is_empty());
    // Should still produce a program structure
    assert_eq!(prog.name, "T");
}

#[test]
fn missing_expression_after_return() {
    // `return end` — return sees `end` which cannot start expression, so bare return
    let (prog, errs) = parse_with_errors("program T; begin return end.");
    assert!(errs.is_empty());
    assert_eq!(prog.body.len(), 1);
}

#[test]
fn empty_body_allowed() {
    let (prog, errs) = parse_with_errors("program T; begin end.");
    assert!(errs.is_empty());
    assert!(prog.body.is_empty());
}

#[test]
fn leading_dot_real_literal_is_rejected() {
    let (_, errs) = parse_with_errors("program T; begin var X: real := .5 end.");
    assert!(!errs.is_empty());
}

#[test]
fn trailing_dot_real_literal_is_rejected() {
    let (_, errs) = parse_with_errors("program T; begin var X: real := 5. end.");
    assert!(!errs.is_empty());
}

#[test]
fn destructure_pattern_requires_binding_identifier() {
    let (_, errs) = parse_with_errors("program T; begin case R of Ok(): X := 1 end end.");
    assert!(!errs.is_empty());
}

#[test]
fn invalid_mutable_statement_reports_statement_start_and_recovers() {
    use fpas_diagnostics::codes::PARSE_INVALID_STATEMENT_START;

    let (_, errs) = parse_with_errors("program T; begin mutable X := X - 10 end.");
    let parse_errors = errs
        .iter()
        .filter_map(|err| match err {
            ParseDiagnostic::Parser(diagnostic) => Some(diagnostic),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        parse_errors.len(),
        1,
        "unexpected parser diagnostics: {parse_errors:#?}"
    );
    assert_eq!(parse_errors[0].code, PARSE_INVALID_STATEMENT_START);
}

#[test]
fn invalid_mutable_statement_recovery_keeps_following_statement() {
    let (program, errs) = parse_with_errors("program T; begin mutable X := X - 10; Y := 1 end.");
    assert!(!errs.is_empty());
    assert_eq!(program.body.len(), 2);
    assert!(matches!(program.body[1], crate::Stmt::Assign { .. }));
}

#[test]
fn multiple_invalid_mutable_statements_recover_until_final_valid_statement() {
    let (program, errs) =
        parse_with_errors("program T; begin mutable X := 1; mutable Y := 2; Z := 3 end.");
    assert!(!errs.is_empty());
    assert_eq!(program.body.len(), 3);
    assert!(matches!(program.body[0], crate::Stmt::Block(_, _)));
    assert!(matches!(program.body[1], crate::Stmt::Block(_, _)));
    assert!(matches!(program.body[2], crate::Stmt::Assign { .. }));
}

#[test]
fn invalid_record_field_start_recovers_without_hanging() {
    let (_, errs) = parse_with_errors("program T; type R = record 123 end; begin end.");
    assert!(!errs.is_empty());
}

#[test]
fn missing_uses_identifier_keeps_non_empty_qualified_id() {
    let (unit, errs) = parse_compilation_unit_with_errors("program T; uses ; begin end.");
    assert!(!errs.is_empty());

    let crate::CompilationUnit::Program(program) = unit else {
        panic!("expected program compilation unit");
    };

    assert_eq!(program.uses.len(), 1);
    assert!(!program.uses[0].parts.is_empty());
}

#[test]
fn missing_uses_segment_after_dot_keeps_placeholder_part() {
    let (unit, errs) = parse_compilation_unit_with_errors("program T; uses Std.; begin end.");
    assert!(!errs.is_empty());

    let crate::CompilationUnit::Program(program) = unit else {
        panic!("expected program compilation unit");
    };

    assert_eq!(program.uses.len(), 1);
    assert_eq!(program.uses[0].parts, vec!["Std", "_error_"]);
}

#[test]
fn empty_uses_entry_before_valid_unit_recovers_next_entry() {
    let (unit, errs) =
        parse_compilation_unit_with_errors("program T; uses , Std.Console; begin end.");
    assert!(!errs.is_empty());

    let crate::CompilationUnit::Program(program) = unit else {
        panic!("expected program compilation unit");
    };

    assert_eq!(program.uses.len(), 2);
    assert_eq!(program.uses[0].parts, vec!["_error_"]);
    assert_eq!(program.uses[1].parts, vec!["Std", "Console"]);
}

#[test]
fn missing_uses_identifier_before_begin_keeps_program_body() {
    let (program, errs) = parse_with_errors("program T; uses begin end.");
    assert!(!errs.is_empty());

    let parse_errors = errs
        .iter()
        .filter_map(|err| match err {
            ParseDiagnostic::Parser(diagnostic) => Some(diagnostic),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(program.body.len(), 0);
    assert_eq!(program.uses.len(), 1);
    assert_eq!(program.uses[0].parts, vec!["_error_"]);
    assert_eq!(
        parse_errors.len(),
        2,
        "unexpected parser diagnostics: {parse_errors:#?}"
    );
}

#[test]
fn missing_uses_identifier_before_declaration_keeps_following_declaration() {
    let (program, errs) = parse_with_errors(
        "program T; uses function Answer(): integer; begin return 42 end; begin end.",
    );
    assert!(!errs.is_empty());
    assert_eq!(program.uses.len(), 1);
    assert_eq!(program.uses[0].parts, vec!["_error_"]);
    assert_eq!(program.declarations.len(), 1);
    assert!(matches!(&program.declarations[0], crate::Decl::Function(_)));
}

// ── Diagnostic code, location and help assertions ───────────────

#[test]
fn expected_token_has_correct_code() {
    use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
    // missing semicolon triggers PARSE_EXPECTED_TOKEN
    let (_, errs) = parse_with_errors("program Hello begin end.");
    assert!(!errs.is_empty());
    let parse_err = errs.iter().find_map(|e| match e {
        ParseDiagnostic::Parser(d) => Some(d),
        _ => None,
    });
    let d = parse_err.expect("expected a parser diagnostic");
    assert_eq!(d.code, PARSE_EXPECTED_TOKEN, "wrong diagnostic code");
    assert_eq!(d.span.line, 1, "wrong line");
    assert!(
        d.help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}

#[test]
fn expected_expression_has_correct_code() {
    use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
    // `X := end` — `end` cannot start an expression
    let (_, errs) = parse_with_errors("program T; begin X := end.");
    let parse_err = errs.iter().find_map(|e| match e {
        ParseDiagnostic::Parser(d) if d.code == PARSE_EXPECTED_EXPRESSION => Some(d),
        _ => None,
    });
    assert!(
        parse_err.is_some(),
        "expected PARSE_EXPECTED_EXPRESSION; got: {errs:#?}"
    );
    let d = parse_err.unwrap();
    assert!(
        d.help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}
