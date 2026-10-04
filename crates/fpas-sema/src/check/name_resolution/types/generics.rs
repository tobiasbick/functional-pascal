use super::super::Checker;
use crate::types::{GenericParamDef, Ty};
use fpas_diagnostics::codes::SEMA_CONSTRAINT_VIOLATION;
use fpas_lexer::Span;

impl Checker {
    /// Check concrete arguments and capabilities of forwarded generic parameters.
    ///
    /// Used during generic function call checking.
    ///
    /// **Documentation:** `docs/pascal/language/functions/generic-routines.md`
    pub(crate) fn validate_constraints(
        &mut self,
        type_params: &[GenericParamDef],
        args: &[Ty],
        span: Span,
    ) {
        for (param, arg) in type_params.iter().zip(args.iter()) {
            if arg.is_error() {
                continue;
            }
            if let Some(constraint) = param.constraint
                && !constraint.satisfied_by_with(arg, |ty| self.resolve_visible_type(ty))
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
                            crate::types::TypeConstraint::Equatable => "structural equality (=, <>) without resource, task, or callable components",
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
