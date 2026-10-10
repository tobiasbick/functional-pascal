//! Calls through array elements and dictionary entries.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`.

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Checks an indexed callable before named-symbol lookup can discard its indexes.
    ///
    /// **Documentation:** `docs/pascal/language/functions/first-class.md`.
    pub(in crate::check) fn try_check_indexed_callable(
        &mut self,
        call_key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> Option<Ty> {
        if !matches!(designator.parts.last(), Some(DesignatorPart::Index(..))) {
            return None;
        }
        let ty = self.check_designator_expr(designator);
        let ty = self.resolve_visible_type(&ty);
        if ty.is_error() {
            self.check_args_only(args);
            return Some(Ty::Error);
        }
        if !matches!(ty, Ty::Function(_) | Ty::Procedure(_)) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Indexed value of type `{ty}` is not callable"),
                "Call only an array element or dictionary entry with a function or procedure type.",
                span,
            );
            self.check_args_only(args);
            return Some(Ty::Error);
        }
        Some(self.check_member_value_call(
            call_key,
            &Self::resolve_designator_name(designator),
            &ty,
            args,
            span,
            allow_procedure_result,
        ))
    }
}
