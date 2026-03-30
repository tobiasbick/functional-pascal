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
use fpas_lexer::{DefineSet, SpannedToken, lex, preprocess};

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
    parse_with_defines(source, &DefineSet::new())
}

/// Parses a full `program` source with the given set of pre-defined conditional
/// symbols.
///
/// Use this variant when the caller needs to pass `{$DEFINE}` symbols from the
/// outside (e.g., command-line `-D` flags).
pub fn parse_with_defines(source: &str, defines: &DefineSet) -> (Program, Vec<ParseDiagnostic>) {
    let (tokens, lex_errors) = lex(source);
    let (tokens, pre_errors) = preprocess(tokens, defines);
    let mut errors: Vec<ParseDiagnostic> = lex_errors
        .into_iter()
        .chain(pre_errors)
        .map(ParseDiagnostic::Lexer)
        .collect();
    let (program, parse_errors) = parser::Parser::new(tokens).parse_program();
    errors.extend(parse_errors.into_iter().map(ParseDiagnostic::Parser));
    (program, errors)
}

pub fn parse_compilation_unit(source: &str) -> (CompilationUnit, Vec<ParseDiagnostic>) {
    parse_compilation_unit_with_defines(source, &DefineSet::new())
}

/// Parses a compilation unit with pre-defined conditional symbols.
pub fn parse_compilation_unit_with_defines(
    source: &str,
    defines: &DefineSet,
) -> (CompilationUnit, Vec<ParseDiagnostic>) {
    let (tokens, lex_errors) = lex(source);
    let (tokens, pre_errors) = preprocess(tokens, defines);
    let mut errors: Vec<ParseDiagnostic> = lex_errors
        .into_iter()
        .chain(pre_errors)
        .map(ParseDiagnostic::Lexer)
        .collect();
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
