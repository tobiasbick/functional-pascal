//! Loss-tolerant parser and public abstract syntax tree for Functional Pascal.
//!
//! Source entry points return both an AST and ordered diagnostics instead of failing fast. Lexer
//! diagnostics precede parser diagnostics, and invalid source produces a partial AST with error
//! placeholders so editors and later diagnostics can keep working.
//!
//! **Language reference:** `docs/pascal/language/README.md` and `docs/specs/grammar.ebnf`.
//!
//! # Examples
//!
//! ```
//! use fpas_parser::parse;
//!
//! let (program, diagnostics) = parse("program Hello; begin end.");
//! assert!(diagnostics.is_empty());
//! assert_eq!(program.name, "Hello");
//! ```

#![deny(missing_docs)]
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

/// A diagnostic emitted while lexing or parsing one source.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseDiagnostic {
    /// Diagnostic emitted by the lexer before parsing begins.
    Lexer(Diagnostic),
    /// Diagnostic emitted while constructing the AST.
    Parser(ParseError),
}

impl ParseDiagnostic {
    /// Returns the common diagnostic representation.
    #[must_use]
    pub fn as_diagnostic(&self) -> &Diagnostic {
        match self {
            Self::Lexer(diagnostic) | Self::Parser(diagnostic) => diagnostic,
        }
    }

    /// Returns the parser-stage diagnostic when this entry is not a lexer error.
    #[must_use]
    pub fn as_parser_error(&self) -> Option<&Diagnostic> {
        match self {
            Self::Parser(diagnostic) => Some(diagnostic),
            Self::Lexer(_) => None,
        }
    }
}

/// Lex `source` into tokens and lexer diagnostics.
fn tokenize(source: &str) -> (Vec<SpannedToken>, Vec<ParseDiagnostic>) {
    let (tokens, lex_errors) = lex(source);
    let errors = lex_errors.into_iter().map(ParseDiagnostic::Lexer).collect();
    (tokens, errors)
}

fn append_parser_errors(
    mut errors: Vec<ParseDiagnostic>,
    parse_errors: Vec<ParseError>,
) -> Vec<ParseDiagnostic> {
    errors.extend(parse_errors.into_iter().map(ParseDiagnostic::Parser));
    errors
}

fn parser_diagnostics(parse_errors: Vec<ParseError>) -> Vec<ParseDiagnostic> {
    parse_errors
        .into_iter()
        .map(ParseDiagnostic::Parser)
        .collect()
}

/// Parses a program source and returns its AST plus ordered diagnostics.
///
/// Lexer diagnostics precede parser diagnostics. Invalid source still returns a partial [`Program`]
/// containing every declaration and statement that recovery could preserve.
///
/// Use [`parse_compilation_unit`] when the source may be either a program or a unit.
///
/// # Recovery
///
/// ```
/// use fpas_parser::parse;
///
/// let (program, diagnostics) = parse("program T; begin A := 1 B := 2 end.");
/// assert_eq!(program.body.len(), 2);
/// assert!(!diagnostics.is_empty());
/// ```
#[must_use]
pub fn parse(source: &str) -> (Program, Vec<ParseDiagnostic>) {
    let (tokens, errors) = tokenize(source);
    let (program, parse_errors) = parser::Parser::new(tokens).parse_program();
    (program, append_parser_errors(errors, parse_errors))
}

/// Parses a program or unit source and returns its partial AST plus ordered diagnostics.
///
/// Lexer diagnostics precede parser diagnostics. A source that does not begin with `program` or
/// `unit` yields a placeholder program and a focused header diagnostic.
#[must_use]
pub fn parse_compilation_unit(source: &str) -> (CompilationUnit, Vec<ParseDiagnostic>) {
    let (tokens, errors) = tokenize(source);
    let (unit, parse_errors) = parser::Parser::new(tokens).parse_compilation_unit();
    (unit, append_parser_errors(errors, parse_errors))
}

/// Parses exactly one Functional Pascal expression.
///
/// Lexer diagnostics precede parser diagnostics. Unsupported callers can inspect the returned
/// [`Expr`] even when recovery produced [`Expr::Error`], but debugger evaluation must reject any
/// non-empty diagnostics before using the tree. Trailing non-trivia tokens produce a focused
/// diagnostic instead of being ignored.
#[must_use]
pub fn parse_expression(source: &str) -> (Expr, Vec<ParseDiagnostic>) {
    let (tokens, errors) = tokenize(source);
    let (expression, parse_errors) = parser::Parser::new(tokens).parse_standalone_expression();
    (expression, append_parser_errors(errors, parse_errors))
}

/// Parses a compilation unit from a pre-lexed token stream.
///
/// Prefer a stream that ends with [`fpas_lexer::Token::Eof`] (as produced by [`fpas_lexer::lex`] or
/// [`fpas_lexer::lex_with_source_id`]). If `Eof` is missing, the parser appends a synthetic one so
/// recovery cannot hang. Its position is the exact end of the last available token; trailing trivia
/// cannot be represented after the original `Eof` token has been removed. An empty stream is
/// accepted and yields a single header diagnostic. Lexer diagnostics are not included; merge them
/// separately.
#[must_use]
pub fn parse_tokens_compilation_unit(
    tokens: Vec<SpannedToken>,
) -> (CompilationUnit, Vec<ParseDiagnostic>) {
    let (unit, parse_errors) = parser::Parser::new(tokens).parse_compilation_unit();
    (unit, parser_diagnostics(parse_errors))
}

#[cfg(test)]
mod tests;
