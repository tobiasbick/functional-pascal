//! Shared value-argument validation for scalar conversions.
//! See `docs/pascal/language/types/subranges.md` and
//! `docs/pascal/language/types/distinct-types.md`.

use super::Checker;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Accepts exactly one positional value, rejecting type declarations as arguments.
    pub(super) fn single_conversion_argument<'a>(
        &mut self,
        name: &str,
        args: &'a [Expr],
        span: Span,
    ) -> Option<&'a Expr> {
        if args.len() != 1 {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Conversion `{name}(...)` expects 1 argument, got {}",
                    args.len()
                ),
                format!("Pass exactly one value, for example `{name}(Value)`."),
                span,
            );
            self.check_args_only(args);
            return None;
        }
        let argument = &args[0];
        if matches!(
            argument,
            Expr::NamedArgument { .. } | Expr::VarArgument { .. }
        ) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Conversion `{name}(...)` takes one positional value"),
                format!("Write `{name}(Value)` without a parameter name or `var` marker."),
                argument.span(),
            );
            self.check_args_only(args);
            return None;
        }
        if self.is_type_designator(argument) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Conversion `{name}(...)` requires a value, found a type name"),
                format!("Pass a value, for example `{name}(Value)`; a type declaration cannot be used as an argument."),
                argument.span(),
            );
            return None;
        }
        Some(argument)
    }
}
