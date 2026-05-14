use super::*;

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
fn missing_expression_after_return() {
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