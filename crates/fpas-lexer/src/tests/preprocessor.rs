use crate::{DefineSet, Token, lex, preprocess};

// Helper: lex + preprocess with given defines, return just the non-EOF tokens.
fn pp(source: &str, defines: &DefineSet) -> Vec<Token> {
    let (tokens, lex_errors) = lex(source);
    assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
    let (out, pre_errors) = preprocess(tokens, defines);
    assert!(pre_errors.is_empty(), "pre errors: {pre_errors:?}");
    out.into_iter()
        .map(|t| t.token)
        .filter(|t| *t != Token::Eof)
        .collect()
}

// Helper: lex + preprocess, return only pre errors (expects lex to be clean).
fn pp_errors(source: &str, defines: &DefineSet) -> Vec<String> {
    let (tokens, lex_errors) = lex(source);
    assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
    let (_, errors) = preprocess(tokens, defines);
    errors.into_iter().map(|e| e.message.clone()).collect()
}

fn empty() -> DefineSet {
    DefineSet::new()
}

fn with(names: &[&str]) -> DefineSet {
    DefineSet::from_iter(names.iter().copied())
}

// ── IFDEF — positive ─────────────────────────────────────────────────────────

#[test]
fn ifdef_defined_symbol_emits_body() {
    assert_eq!(
        pp("{$IFDEF DEBUG} 1 {$ENDIF}", &with(&["DEBUG"])),
        vec![Token::Integer(1)]
    );
}

#[test]
fn ifdef_undefined_symbol_suppresses_body() {
    assert!(pp("{$IFDEF DEBUG} 1 {$ENDIF}", &empty()).is_empty());
}

#[test]
fn ifdef_else_branch_emitted_when_not_defined() {
    assert_eq!(
        pp("{$IFDEF DEBUG} 1 {$ELSE} 2 {$ENDIF}", &empty()),
        vec![Token::Integer(2)]
    );
}

#[test]
fn ifdef_else_branch_suppressed_when_defined() {
    assert_eq!(
        pp("{$IFDEF DEBUG} 1 {$ELSE} 2 {$ENDIF}", &with(&["DEBUG"])),
        vec![Token::Integer(1)]
    );
}

// ── IFNDEF ───────────────────────────────────────────────────────────────────

#[test]
fn ifndef_emits_body_when_not_defined() {
    assert_eq!(
        pp("{$IFNDEF RELEASE} 3 {$ENDIF}", &empty()),
        vec![Token::Integer(3)]
    );
}

#[test]
fn ifndef_suppresses_body_when_defined() {
    assert!(pp("{$IFNDEF RELEASE} 3 {$ENDIF}", &with(&["RELEASE"])).is_empty());
}

// ── DEFINE / UNDEF ───────────────────────────────────────────────────────────

#[test]
fn define_in_source_enables_later_ifdef() {
    assert_eq!(
        pp("{$DEFINE FOO} {$IFDEF FOO} 5 {$ENDIF}", &empty()),
        vec![Token::Integer(5)]
    );
}

#[test]
fn undef_in_source_disables_later_ifdef() {
    assert!(
        pp(
            "{$DEFINE FOO} {$UNDEF FOO} {$IFDEF FOO} 5 {$ENDIF}",
            &empty()
        )
        .is_empty()
    );
}

#[test]
fn define_only_in_active_branch_does_not_leak() {
    // DEFINE inside a suppressed branch must not affect later code.
    let src = "{$IFDEF GHOST} {$DEFINE FOO} {$ENDIF} {$IFDEF FOO} 99 {$ENDIF}";
    assert!(pp(src, &empty()).is_empty());
}

// ── Case-insensitivity ───────────────────────────────────────────────────────

#[test]
fn directive_keywords_are_case_insensitive() {
    assert_eq!(
        pp("{$ifdef DEBUG} 7 {$endif}", &with(&["DEBUG"])),
        vec![Token::Integer(7)]
    );
}

#[test]
fn symbol_names_are_case_insensitive() {
    // Defined as lowercase, tested as uppercase.
    assert_eq!(
        pp("{$IFDEF debug} 8 {$ENDIF}", &with(&["debug"])),
        vec![Token::Integer(8)]
    );
}

// ── Nesting ───────────────────────────────────────────────────────────────────

#[test]
fn nested_ifdef_both_true() {
    let src = "{$IFDEF A} {$IFDEF B} 10 {$ENDIF} {$ENDIF}";
    assert_eq!(
        pp(src, &with(&["A", "B"])),
        vec![Token::Integer(10)]
    );
}

#[test]
fn nested_ifdef_outer_false_skips_inner() {
    let src = "{$IFDEF A} {$IFDEF B} 10 {$ENDIF} {$ENDIF}";
    // B is defined but A is not — the inner block must not be processed.
    assert!(pp(src, &with(&["B"])).is_empty());
}

#[test]
fn nested_ifdef_inner_false() {
    let src = "{$IFDEF A} {$IFDEF B} 10 {$ELSE} 20 {$ENDIF} {$ENDIF}";
    assert_eq!(
        pp(src, &with(&["A"])),
        vec![Token::Integer(20)]
    );
}

#[test]
fn deeply_nested_ifdef() {
    let src = "{$IFDEF A}{$IFDEF B}{$IFDEF C} 42 {$ENDIF}{$ENDIF}{$ENDIF}";
    assert_eq!(
        pp(src, &with(&["A", "B", "C"])),
        vec![Token::Integer(42)]
    );
}

// ── Code surrounds ────────────────────────────────────────────────────────────

#[test]
fn code_before_and_after_directive_block() {
    let src = "1 {$IFDEF X} 2 {$ENDIF} 3";
    assert_eq!(
        pp(src, &empty()),
        vec![Token::Integer(1), Token::Integer(3)]
    );
}

#[test]
fn code_before_and_after_with_x_defined() {
    let src = "1 {$IFDEF X} 2 {$ENDIF} 3";
    assert_eq!(
        pp(src, &with(&["X"])),
        vec![Token::Integer(1), Token::Integer(2), Token::Integer(3)]
    );
}

// ── Error cases ───────────────────────────────────────────────────────────────

#[test]
fn else_without_ifdef_is_error() {
    let msgs = pp_errors("{$ELSE}", &empty());
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].contains("without a matching"),
        "unexpected message: {}",
        msgs[0]
    );
}

#[test]
fn endif_without_ifdef_is_error() {
    let msgs = pp_errors("{$ENDIF}", &empty());
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].contains("without a matching"),
        "unexpected message: {}",
        msgs[0]
    );
}

#[test]
fn unclosed_ifdef_is_error() {
    let msgs = pp_errors("{$IFDEF FOO} 1", &empty());
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].contains("Unclosed"),
        "unexpected message: {}",
        msgs[0]
    );
}

#[test]
fn duplicate_else_is_error() {
    let msgs = pp_errors("{$IFDEF FOO} 1 {$ELSE} 2 {$ELSE} 3 {$ENDIF}", &empty());
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].contains("Duplicate"),
        "unexpected message: {}",
        msgs[0]
    );
}

#[test]
fn include_in_single_file_mode_is_error() {
    // An INCLUDE directive in the active branch must produce exactly one error.
    let (tokens, _) = lex("{$INCLUDE helpers.fpas}");
    let (out, errors) = preprocess(tokens, &empty());
    assert!(
        out.iter().all(|t| t.token == Token::Eof),
        "expected no real tokens"
    );
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].message.contains("single-file mode"),
        "unexpected message: {}",
        errors[0].message
    );
}

#[test]
fn include_in_inactive_branch_is_suppressed() {
    // INCLUDE inside a false IFDEF branch must NOT produce an error.
    let msgs = pp_errors("{$IFDEF GHOST} {$INCLUDE foo.fpas} {$ENDIF}", &empty());
    assert!(msgs.is_empty(), "expected no errors, got: {msgs:?}");
}

#[test]
fn unknown_directive_in_active_branch_is_error() {
    let msgs = pp_errors("{$R+}", &empty());
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].contains("Unknown compiler directive"),
        "unexpected message: {}",
        msgs[0]
    );
}

#[test]
fn unknown_directive_in_inactive_branch_is_ignored() {
    let msgs = pp_errors("{$IFDEF GHOST} {$R+} {$ENDIF}", &empty());
    assert!(msgs.is_empty(), "expected no errors, got: {msgs:?}");
}

// ── Edge cases ────────────────────────────────────────────────────────────────

#[test]
fn empty_source_no_errors() {
    assert!(pp("", &empty()).is_empty());
}

#[test]
fn no_directives_passes_through_unchanged() {
    let src = "1 2 3";
    assert_eq!(
        pp(src, &empty()),
        vec![Token::Integer(1), Token::Integer(2), Token::Integer(3)]
    );
}

#[test]
fn multiple_defines_independent() {
    assert_eq!(
        pp(
            "{$IFDEF A} 1 {$ENDIF} {$IFDEF B} 2 {$ENDIF}",
            &with(&["A"])
        ),
        vec![Token::Integer(1)]
    );
}

#[test]
fn ifdef_immediately_followed_by_endif_emits_nothing() {
    assert!(pp("{$IFDEF DEBUG}{$ENDIF}", &with(&["DEBUG"])).is_empty());
}
