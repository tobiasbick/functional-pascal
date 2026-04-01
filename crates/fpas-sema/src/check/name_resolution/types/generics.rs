use super::super::Checker;
use crate::types::{GenericParamDef, Ty};
use fpas_diagnostics::codes::SEMA_CONSTRAINT_VIOLATION;
use fpas_lexer::Span;

impl Checker {
    /// Check that each concrete type argument satisfies its parameter's constraint.
    ///
    /// Used during generic function call checking.
    ///
    /// **Documentation:** `docs/pascal/04-functions.md` (Generic Functions)
    pub(crate) fn validate_constraints(
        &mut self,
        type_params: &[GenericParamDef],
        args: &[Ty],
        span: Span,
    ) {
        for (param, arg) in type_params.iter().zip(args.iter()) {
            // Skip validation for error types and unresolved generic params.
            if arg.is_error() || matches!(arg, Ty::GenericParam(..)) {
                continue;
            }
            if let Some(constraint) = param.constraint
                && !constraint.satisfied_by(arg)
            {
                self.error_with_code(
                    SEMA_CONSTRAINT_VIOLATION,
                    format!(
                        "Type `{arg}` does not satisfy constraint `{}` on parameter `{}`",
                        constraint.display_name(),
                        param.name,
                    ),
                    format!(
                        "The `{}` constraint requires a type that supports {}.",
                        constraint.display_name(),
                        match constraint {
                            crate::types::TypeConstraint::Comparable =>
                                "comparison operators (=, <>, <, >, <=, >=)",
                            crate::types::TypeConstraint::Numeric =>
                                "arithmetic operators (+, -, *, /, div, mod)",
                            crate::types::TypeConstraint::Printable => "string conversion",
                        },
                    ),
                    span,
                );
            }
        }
    }
}
