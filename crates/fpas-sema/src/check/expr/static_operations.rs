//! Checked operations in static initializers, labels and record defaults.
//!
//! **Documentation:** `docs/pascal/language/basics/constants.md`.

use crate::check::Checker;
use fpas_diagnostics::codes::SEMA_INVALID_STATIC_OPERATION;
use fpas_parser::Expr;

impl Checker {
    /// Diagnose reached static failures, including aggregate operands of lazy expressions.
    pub(in crate::check) fn validate_static_operations(&mut self, expression: &Expr) {
        if let Err(error) = self.try_evaluate_static_value(expression) {
            self.error_with_code(
                SEMA_INVALID_STATIC_OPERATION,
                error.operation.to_string(),
                "Static integer operations must fit signed 64-bit values and use a nonzero divisor.",
                error.span,
            );
        }
    }
}
