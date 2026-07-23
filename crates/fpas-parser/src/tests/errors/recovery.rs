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

#[test]
fn truncated_token_stream_without_eof_does_not_hang() {
    use crate::parse_tokens_compilation_unit;
    use fpas_lexer::{Span, SpannedToken, Token, lex};

    let (mut tokens, _) = lex("program T; begin end.");
    // Drop the trailing Eof that lex always appends.
    if matches!(tokens.last().map(|t| &t.token), Some(Token::Eof)) {
        tokens.pop();
    }
    assert!(
        !matches!(tokens.last().map(|t| &t.token), Some(Token::Eof)),
        "fixture must omit trailing Eof"
    );

    let (unit, errors) = parse_tokens_compilation_unit(tokens);
    assert!(matches!(unit, crate::CompilationUnit::Program(_)));
    // Synthetic Eof lets parsing complete; may or may not have extra diagnostics.
    let _ = errors;

    // Also exercise a minimal truncated stream that previously could spin in skip_to_eof.
    let (unit2, _) = parse_tokens_compilation_unit(vec![SpannedToken {
        token: Token::Const,
        span: Span {
            offset: 0,
            length: 5,
            line: 1,
            column: 1,
            source_id: 0,
        },
    }]);
    assert!(matches!(unit2, crate::CompilationUnit::Program(_)));
}

#[test]
fn case_missing_semicolon_between_arms_keeps_following_arms() {
    let (program, errs) =
        parse_with_errors("program T; begin case X of 1: A := 1 2: A := 2; 3: A := 3 end end.");
    assert!(!errs.is_empty());
    match &program.body[0] {
        crate::Stmt::Case { arms, .. } => {
            assert!(
                arms.len() >= 2,
                "expected recovery to keep later arms, got {arms:#?}"
            );
        }
        other => panic!("expected Case, got {other:#?}"),
    }
}

#[test]
fn trailing_semicolon_in_param_list_does_not_invent_extra_param() {
    let (program, errs) = parse_with_errors(
        "program T; function F(X: integer;): integer; begin return X end; begin end.",
    );
    assert!(!errs.is_empty());
    match &program.declarations[0] {
        crate::Decl::Function(f) => {
            assert_eq!(
                f.params.len(),
                1,
                "expected one real param, got {params:#?}",
                params = f.params
            );
        }
        other => panic!("expected Function, got {other:#?}"),
    }
}
