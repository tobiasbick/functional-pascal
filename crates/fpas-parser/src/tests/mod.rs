use crate::{CompilationUnit, ParseDiagnostic, Program, Unit, parse, parse_compilation_unit};

pub fn parse_ok(src: &str) -> Program {
    let (program, errors) = parse(src);
    assert!(errors.is_empty(), "unexpected errors: {errors:#?}");
    program
}

pub fn parse_with_errors(src: &str) -> (Program, Vec<ParseDiagnostic>) {
    parse(src)
}

pub fn parse_unit_ok(src: &str) -> Unit {
    let (unit, errors) = parse_compilation_unit(src);
    assert!(errors.is_empty(), "unexpected errors: {errors:#?}");
    match unit {
        CompilationUnit::Unit(unit) => unit,
        CompilationUnit::Program(_) => panic!("expected unit compilation unit"),
    }
}

pub fn parse_compilation_unit_with_errors(src: &str) -> (CompilationUnit, Vec<ParseDiagnostic>) {
    parse_compilation_unit(src)
}

mod decl;
mod errors;
mod expr;
mod integration;
mod stmt;
