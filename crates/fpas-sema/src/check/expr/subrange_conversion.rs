//! Explicit checked conversion to an integer subrange.
//!
//! **Documentation:** `docs/pascal/language/types/subranges.md`

use super::Checker;
use crate::types::{SubrangeTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_unit::interface::ConstantValue;

impl Checker {
    /// Checks one integer value and rejects a known value outside the target bounds.
    pub(in crate::check) fn check_subrange_conversion(
        &mut self,
        key: usize,
        name: &str,
        range: &SubrangeTy,
        args: &[Expr],
        span: Span,
    ) -> Ty {
        let target = Ty::Subrange(range.clone());
        let Some(argument) = self.single_conversion_argument(name, args, span) else {
            return target;
        };
        let actual = self.check_expr(argument);
        if actual.is_error() {
            return target;
        }
        if !Ty::Integer.assignment_compatible_with(&actual) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{name}(...)` requires an integer or subrange value, found `{actual}`"),
                format!("Convert an integer explicitly, for example `{name}(Value)`."),
                argument.span(),
            );
            return target;
        }
        if self.const_expr_is_compile_time_known(argument)
            && let Some(ConstantValue::Integer(value)) = self.scalar_constant_value(argument)
            && !range.contains(value)
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Value {value} is outside subrange `{name}` ({}..{})",
                    range.lower, range.upper
                ),
                format!(
                    "Use a value in the inclusive range {}..{}.",
                    range.lower, range.upper
                ),
                argument.span(),
            );
        }
        self.subrange_conversions.insert(key, range.clone());
        target
    }
}
