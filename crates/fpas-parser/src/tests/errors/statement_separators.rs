use super::*;
use crate::Stmt;
use fpas_diagnostics::codes::PARSE_INVALID_STATEMENT_START;

#[test]
fn program_body_missing_separator_keeps_following_statement() {
    let (program, errors) = parse_with_errors("program T; begin A := 1 B := 2 end.");

    assert_eq!(program.body.len(), 2, "body: {:#?}", program.body);
    assert_eq!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .filter(|error| error.message.contains("between statements"))
            .count(),
        1,
        "diagnostics: {errors:#?}"
    );
}

#[test]
fn statement_list_boundaries_do_not_require_separators() {
    for source in [
        "program T; begin A := 1 end.",
        "program T; begin if C then A := 1 else B := 2 end.",
        "program T; begin repeat A := 1 until Done end.",
    ] {
        let (_, errors) = parse_with_errors(source);
        assert!(
            errors.is_empty(),
            "source: {source}; diagnostics: {errors:#?}"
        );
    }
}

#[test]
fn if_branch_blocks_missing_separators_keep_following_statements() {
    let (program, errors) = parse_with_errors(
        "program T; begin if C then begin A := 1 B := 2 end else begin C := 3 D := 4 end end.",
    );

    let Stmt::If {
        then_branch,
        else_branch: Some(else_branch),
        ..
    } = &program.body[0]
    else {
        panic!("expected if statement, got {:#?}", program.body[0]);
    };
    let Stmt::Block(then_statements, _) = then_branch.as_ref() else {
        panic!("expected then block, got {then_branch:#?}");
    };
    let Stmt::Block(else_statements, _) = else_branch.as_ref() else {
        panic!("expected else block, got {else_branch:#?}");
    };
    assert_eq!(then_statements.len(), 2);
    assert_eq!(else_statements.len(), 2);
    assert_eq!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .filter(|error| error.message.contains("between statements"))
            .count(),
        2,
        "diagnostics: {errors:#?}"
    );
}

#[test]
fn repeat_missing_separator_keeps_following_statement() {
    let (program, errors) =
        parse_with_errors("program T; begin repeat A := 1 B := 2 until Done end.");

    let Stmt::Repeat { body, .. } = &program.body[0] else {
        panic!("expected repeat statement, got {:#?}", program.body[0]);
    };
    assert_eq!(body.len(), 2, "repeat body: {body:#?}");
    assert_eq!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .filter(|error| error.message.contains("between statements"))
            .count(),
        1,
        "diagnostics: {errors:#?}"
    );
}

#[test]
fn case_else_missing_separator_keeps_following_statement() {
    let (program, errors) =
        parse_with_errors("program T; begin case X of 1: A := 1; else B := 2 C := 3 end end.");

    let Stmt::Case {
        else_body: Some(else_body),
        ..
    } = &program.body[0]
    else {
        panic!("expected case statement, got {:#?}", program.body[0]);
    };
    assert_eq!(else_body.len(), 2, "case else body: {else_body:#?}");
    assert_eq!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .filter(|error| error.message.contains("between statements"))
            .count(),
        1,
        "diagnostics: {errors:#?}"
    );
}

#[test]
fn separator_recovery_skips_invalid_tokens_before_next_statement() {
    let (program, errors) = parse_with_errors("program T; begin A := 1 : B := 2 end.");

    assert_eq!(program.body.len(), 2, "body: {:#?}", program.body);
    assert_eq!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .filter(|error| error.message.contains("between statements"))
            .count(),
        1,
        "diagnostics: {errors:#?}"
    );
}

#[test]
fn separator_recovery_resumes_normal_statement_parsing_after_found_semicolon() {
    let (program, errors) = parse_with_errors("program T; begin A := 1 : ; : ; B := 2 end.");
    let parser_errors: Vec<_> = errors
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .collect();

    assert_eq!(program.body.len(), 3, "body: {:#?}", program.body);
    assert_eq!(
        parser_errors
            .iter()
            .filter(|error| error.message.contains("between statements"))
            .count(),
        1,
        "diagnostics: {parser_errors:#?}"
    );
    assert_eq!(
        parser_errors
            .iter()
            .filter(|error| error.code == PARSE_INVALID_STATEMENT_START)
            .count(),
        1,
        "diagnostics: {parser_errors:#?}"
    );
    assert!(matches!(program.body.last(), Some(Stmt::Assign { .. })));
}
