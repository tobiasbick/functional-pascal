//! Routine-entry and return contract clauses.
//!
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

use super::Expr;
use fpas_lexer::Span;

/// A Boolean `requires` clause checked on every routine entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RequiresClause {
    /// Parsed Boolean condition.
    pub condition: Expr,
    /// Original clause text, including its keyword and terminator.
    pub text: String,
    /// Source span of the complete clause.
    pub span: Span,
}

/// A Boolean `ensures` clause checked on every ordinary or propagated return.
#[derive(Debug, Clone, PartialEq)]
pub struct EnsuresClause {
    /// Optional clause-local name and declaration span of the complete return value.
    pub result_name: Option<(String, Span)>,
    /// Parsed Boolean condition.
    pub condition: Expr,
    /// Original clause text, including its keyword and terminator.
    pub text: String,
    /// Source span of the complete clause.
    pub span: Span,
}
