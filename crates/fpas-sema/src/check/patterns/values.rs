//! Shared static-value requirements for scalar labels and recursive payload patterns.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/scalar-labels.md`.

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION;
use fpas_parser::Expr;

impl Checker {
    /// Reject computed labels and endpoints without duplicating primary type errors.
    pub(in crate::check) fn require_static_pattern_value(
        &mut self,
        expression: &Expr,
        actual: &Ty,
        range_endpoint: bool,
    ) {
        if actual.is_error() || self.const_expr_is_compile_time_known(expression) {
            return;
        }
        let (message, help) = if range_endpoint {
            (
                "Pattern ranges require static endpoints",
                "Use literal or static-constant bounds; put computed comparisons in an if guard.",
            )
        } else {
            (
                "Pattern values must be literals or static constants",
                "Use `const Name` to bind the matched value, or put a computed comparison in an if guard.",
            )
        };
        self.error_with_code(
            SEMA_NON_CONSTANT_EXPRESSION,
            message,
            help,
            expression.span(),
        );
    }
}
