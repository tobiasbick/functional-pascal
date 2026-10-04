//! Value-producing decisions with one expression in each branch.

use super::{CaseArm, Expr};
use fpas_lexer::Span;

/// Conditional value expression with a required fallback.
///
/// **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`.
#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    /// Boolean condition evaluated first.
    pub condition: Expr,
    /// Value selected when the first condition succeeds.
    pub then_value: Expr,
    /// Additional conditions and their selected values in written order.
    pub elsif_values: Vec<(Expr, Expr)>,
    /// Value selected when every condition fails.
    pub else_value: Expr,
    /// Complete source span, including `end if`.
    pub span: Span,
}

/// Pattern-selected value expression with statement-case pattern rules.
///
/// **Documentation:** `docs/pascal/language/pattern-matching/README.md`.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseExpr {
    /// Value evaluated once before testing arms.
    pub value: Expr,
    /// Pattern labels, guards, and selected expressions in written order.
    pub arms: Vec<CaseArm<Expr>>,
    /// Fallback value for an open scalar domain.
    pub else_value: Option<Expr>,
    /// Complete source span, including `end case`.
    pub span: Span,
}
