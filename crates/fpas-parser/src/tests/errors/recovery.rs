use super::*;

#[test]
fn error_recovery_continues() {
    let (prog, errs) = parse_with_errors("program T; begin X := 1; Y := end.");
    assert!(!errs.is_empty());
    assert_eq!(prog.name, "T");
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