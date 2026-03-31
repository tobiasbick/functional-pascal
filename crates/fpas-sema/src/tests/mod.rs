use crate::{SemaError, analyze};
use fpas_parser::parse;

pub fn check_ok(src: &str) {
    let (program, parse_errors) = parse(src);
    assert!(
        parse_errors.is_empty(),
        "unexpected parse errors: {parse_errors:#?}"
    );
    let sema_errors = analyze(&program);
    assert!(
        sema_errors.is_empty(),
        "unexpected sema errors: {sema_errors:#?}"
    );
}

pub fn check_errors(src: &str) -> Vec<SemaError> {
    let (program, parse_errors) = parse(src);
    assert!(
        parse_errors.is_empty(),
        "unexpected parse errors: {parse_errors:#?}"
    );
    let sema_errors = analyze(&program);
    assert!(!sema_errors.is_empty(), "expected sema errors but got none");
    sema_errors
}

mod decl;
mod expr;
mod generic_methods;
mod integration;
mod stmt;
