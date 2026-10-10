//! Exhaustiveness diagnostics and closed-enum catch-all restrictions.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`

use super::{Checker, coverage::Pat};
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_CLOSED_ENUM_ELSE, SEMA_NON_EXHAUSTIVE_CASE};
use fpas_lexer::Span;

const EXPLICIT_ARMS_HINT: &str = "Use explicit `when` arms; write `null;` for variants with no action. If only one variant matters, use `if Value is Pattern then ... end if;`. Guarded arms do not count toward coverage.";

impl Checker {
    /// Rejects enum catch-alls and checks unguarded coverage without changing scalar cases.
    pub(super) fn check_case_exhaustiveness(
        &mut self,
        case_ty: &Ty,
        rows: &[Vec<Pat>],
        coverage_valid: bool,
        has_else: bool,
        span: Span,
    ) {
        let is_closed_enum = self.resolve_enum_ty(case_ty).is_some()
            || matches!(case_ty, Ty::Option(_) | Ty::Result(_, _));
        let missing = if coverage_valid {
            self.missing_patterns(rows, case_ty)
        } else {
            Vec::new()
        };
        if has_else && is_closed_enum {
            let (message, hint) = if coverage_valid && missing.is_empty() {
                (
                    "`else` is not allowed in a case over a closed enum".to_string(),
                    "This case already covers every variant. Remove the redundant `else` branch.",
                )
            } else {
                let message = if missing.is_empty() {
                    "`else` is not allowed in a case over a closed enum".to_string()
                } else {
                    format!(
                        "`else` is not allowed in a case over a closed enum: missing {}",
                        missing.join(", ")
                    )
                };
                (message, EXPLICIT_ARMS_HINT)
            };
            self.error_with_code(SEMA_CLOSED_ENUM_ELSE, message, hint, span);
        } else if coverage_valid && !missing.is_empty() {
            self.error_with_code(
                SEMA_NON_EXHAUSTIVE_CASE,
                format!("Non-exhaustive case: missing {}", missing.join(", ")),
                EXPLICIT_ARMS_HINT,
                span,
            );
        }
    }
}
