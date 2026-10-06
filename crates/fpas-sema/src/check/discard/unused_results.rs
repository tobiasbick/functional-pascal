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
            let safe = self.call_result_is_task_free(
                crate::designator_lookup_key(designator),
                designator,
                &ty,
            );
            self.check_unused_result(&ty, safe, span);
        }
    }

    /// Suggests explicit discard only when the existing operand rules permit it.
    pub(crate) fn check_unused_result(&mut self, ty: &Ty, safe: bool, span: Span) {
        let ty = self.resolve_visible_type(ty);
        if matches!(ty, Ty::Unit | Ty::Error) {
            return;
        }
        let hint = match (&ty, safe) {
            (Ty::Result(_, _), true) => {
                "Handle the result with `case`, propagate errors with `try` while consuming the success value, or explicitly ignore it with `discard Call();`."
            }
            (Ty::Result(_, _), false) => {
                "Handle the result with `case` or propagate errors with `try` while retaining and consuming its task handles or callable captures."
            }
            (_, true) => {
                "Assign or return the value, pass it to another call, or explicitly ignore it with `discard Call();`."
            }
            (_, false) => {
                "Retain and consume the value; its type or unverified callable captures prevent explicit discard."
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
