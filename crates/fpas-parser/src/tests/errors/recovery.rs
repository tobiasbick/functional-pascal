use super::*;

#[test]
fn record_initializers_preserve_recovery_boundaries_without_repeating_errors() {
    for expression in [
        "record then end",
        "record ) end",
        "P with then end",
        "P with ) end",
    ] {
        let source = format!("program P; begin var Value: integer := {expression}; end.");
        let (_, diagnostics) = parse_with_errors(&source);
        assert!(!diagnostics.is_empty(), "{source}");
        assert!(diagnostics.len() < 16, "{source}: {diagnostics:#?}");
    }
    for expression in ["Point( X := 1 )", "P with X := 1; end with"] {
        let source = format!("program P; begin var Value: integer := {expression}; end.");
        let (_, diagnostics) = parse_with_errors(&source);
        assert!(diagnostics.is_empty(), "{source}: {diagnostics:#?}");
    }
}

#[test]
fn error_recovery_continues() {
    let (prog, errs) = parse_with_errors("program T; begin X := 1; Y := end.");
    assert!(!errs.is_empty());
    assert_eq!(prog.name, "T");
}

#[test]
fn retired_binding_recovery_keeps_following_statements() {
    let (program, errors) = parse_with_errors(
        "program T; begin mutable var X: integer := 1; mutable var Y: integer := 2; Z := 3; end.",
    );
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert_eq!(program.body.len(), 3);
    assert!(matches!(program.body[0], crate::Stmt::Var(_)));
    assert!(matches!(program.body[1], crate::Stmt::Var(_)));
    assert!(matches!(program.body[2], crate::Stmt::Assign { .. }));
}

#[test]
fn invalid_record_field_start_recovers_without_hanging() {
    let (_, errs) = parse_with_errors("program T; type R = record 123 end record; begin end.");
    assert!(!errs.is_empty());
}

#[test]
fn truncated_token_stream_without_eof_does_not_hang() {
    use crate::parse_tokens_compilation_unit;
    use fpas_lexer::{SourcePosition, Span, SpannedToken, Token, lex};

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
        end: SourcePosition {
            offset: 5,
            line: 1,
            column: 6,
        },
    }]);
    assert!(matches!(unit2, crate::CompilationUnit::Program(_)));
}

#[test]
fn case_missing_semicolon_between_arms_keeps_following_arms() {
    let (program, errs) = parse_with_errors(
        "program T; begin case X of when 1: A := 1 when 2: A := 2; when 3: A := 3; end case; end.",
    );
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
        "program T; function F(X: integer;): integer; begin return X; end function; begin end.",
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

#[test]
fn empty_declaration_sections_report_errors_and_recover() {
    for source in [
        "program T; const begin end.",
        "program T; var begin end.",
        "program T; mutable var begin end.",
        "program T; type begin end.",
        "program T; type E = enum end enum; begin end.",
        "program T; begin case 1 of end case; end.",
    ] {
        let (_, errors) = parse_with_errors(source);
        assert!(!errors.is_empty(), "expected parser error for `{source}`");
    }
}

#[test]
fn empty_const_section_keeps_following_var_declaration() {
    let (program, errors) = parse_with_errors("program T; const var X: integer := 1; begin end.");

    assert!(!errors.is_empty());
    assert_eq!(program.declarations.len(), 1);
    assert!(matches!(program.declarations[0], crate::Decl::Var(_)));
}

#[test]
fn expression_recovery_keeps_record_end_and_following_statement() {
    use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;

    let (program, errors) =
        parse_with_errors("program T; begin X := record Field := end; Y := 1; end.");

    assert!(errors.iter().any(|error| {
        error
            .as_parser_error()
            .is_some_and(|error| error.code == PARSE_EXPECTED_EXPRESSION)
    }));
    assert_eq!(program.body.len(), 2, "program AST: {program:#?}");
    assert!(matches!(program.body[1], crate::Stmt::Assign { .. }));
}

#[test]
fn invalid_top_level_static_keeps_recovered_routine() {
    use fpas_diagnostics::codes::PARSE_INVALID_STATIC_PLACEMENT;

    let (program, errors) = parse_with_errors(
        "program T; static function Foo(): integer; begin return 1; end function; begin end.",
    );

    assert!(errors.iter().any(|error| {
        error
            .as_parser_error()
            .is_some_and(|error| error.code == PARSE_INVALID_STATIC_PLACEMENT)
    }));
    assert_eq!(program.declarations.len(), 1);
    match &program.declarations[0] {
        crate::Decl::Function(function) => assert_eq!(function.name, "Foo"),
        other => panic!("expected recovered function, got {other:#?}"),
    }
}
