#![cfg_attr(
    test,
    expect(
        clippy::approx_constant,
        reason = "parser tests use literal Pascal fixtures and direct numeric assertions"
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "parser tests use expect to fail fast on missing diagnostics"
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::panic,
        reason = "parser tests use explicit pattern mismatch panics to keep AST assertions readable"
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::unwrap_used,
        reason = "parser tests use unwrap in a few direct diagnostic assertions"
    )
)]

mod ast;
mod error;
mod parser;

pub use ast::*;
pub use error::ParseError;

use fpas_diagnostics::Diagnostic;
use fpas_lexer::{SpannedToken, lex};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseDiagnostic {
    Lexer(Diagnostic),
    Parser(ParseError),
}

impl ParseDiagnostic {
    #[must_use]
    pub fn as_diagnostic(&self) -> &Diagnostic {
        match self {
            Self::Lexer(diagnostic) | Self::Parser(diagnostic) => diagnostic,
        }
    }
}

pub fn parse(source: &str) -> (Program, Vec<ParseDiagnostic>) {
    let (tokens, lex_errors) = lex(source);
    let mut errors: Vec<ParseDiagnostic> =
        lex_errors.into_iter().map(ParseDiagnostic::Lexer).collect();
    let (program, parse_errors) = parser::Parser::new(tokens).parse_program();
    errors.extend(parse_errors.into_iter().map(ParseDiagnostic::Parser));
    (program, errors)
}

pub fn parse_compilation_unit(source: &str) -> (CompilationUnit, Vec<ParseDiagnostic>) {
    let (tokens, lex_errors) = lex(source);
    let mut errors: Vec<ParseDiagnostic> =
        lex_errors.into_iter().map(ParseDiagnostic::Lexer).collect();
    let (unit, parse_errors) = parser::Parser::new(tokens).parse_compilation_unit();
    errors.extend(parse_errors.into_iter().map(ParseDiagnostic::Parser));
    (unit, errors)
}

pub fn parse_tokens(tokens: Vec<SpannedToken>) -> (Program, Vec<ParseDiagnostic>) {
    let (program, parse_errors) = parser::Parser::new(tokens).parse_program();
    (
        program,
        parse_errors
            .into_iter()
            .map(ParseDiagnostic::Parser)
            .collect(),
    )
}

pub fn parse_tokens_compilation_unit(
    tokens: Vec<SpannedToken>,
) -> (CompilationUnit, Vec<ParseDiagnostic>) {
    let (unit, parse_errors) = parser::Parser::new(tokens).parse_compilation_unit();
    (
        unit,
        parse_errors
            .into_iter()
            .map(ParseDiagnostic::Parser)
            .collect(),
    )
}

#[cfg(test)]
mod tests;
