//! Required consumption of function results in statement position.
//!
//! **Documentation:** `docs/pascal/language/functions/discard.md`

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_UNUSED_FUNCTION_RESULT;
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};

impl Checker {
    /// Checks a standalone call and diagnoses only a successfully resolved value result.
    pub(crate) fn check_unused_call_stmt(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) {
        let previous_error_count = self.errors.len();
        let ty = self.check_call_stmt(designator, args, span);
        if self.errors.len() == previous_error_count {
            self.check_unused_result(&ty, span);
        }
    }

    /// Suggests ordinary value consumption, including explicit discard.
    pub(crate) fn check_unused_result(&mut self, ty: &Ty, span: Span) {
        let ty = self.resolve_visible_type(ty);
        if matches!(ty, Ty::Unit | Ty::Error) {
            return;
        }
        let hint = match &ty {
            Ty::Result(_, _) => {
                "Handle the result with `case`, propagate errors with `try` while consuming the success value, or explicitly ignore it with `discard Call();`."
            }
            _ => {
                "Assign or return the value, pass it to another call, or explicitly ignore it with `discard Call();`."
            }
        };
        self.error_with_code(
            SEMA_UNUSED_FUNCTION_RESULT,
            format!("Unused function result of type `{ty}`"),
            hint,
            span,
        );
    }
}
