//! One selector-domain policy for exhaustive statements and expressions.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`

use super::{Checker, coverage::Pat};
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_CASE_ELSE_FORBIDDEN, SEMA_NON_EXHAUSTIVE_CASE};
use fpas_lexer::Span;

const EXPLICIT_ARMS_HINT: &str = "Add explicit `when` arms for the missing values or variants; write `null;` for cases with no action. Use `if` for selective handling, for example `if Value is Pattern then ... end if;` for a variant. Guarded arms do not count toward coverage.";

const EXPRESSION_ARMS_HINT: &str = "Add a `when` arm with a result for each missing value or variant. Guarded arms do not count toward coverage.";

/// Whether coverage diagnostics describe a `case` statement or a `case` expression.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CaseForm {
    Statement,
    Expression,
}

impl Checker {
    /// Apply identical finite-domain and fallback rules to both case positions.
    pub(super) fn check_case_exhaustiveness(
        &mut self,
        case_ty: &Ty,
        rows: &[Vec<Pat>],
        coverage_valid: bool,
        has_else: bool,
        form: CaseForm,
        span: Span,
    ) {
        let (noun, arms_hint) = match form {
            CaseForm::Statement => ("case", EXPLICIT_ARMS_HINT),
            CaseForm::Expression => ("case expression", EXPRESSION_ARMS_HINT),
        };
        let resolved = self.resolve_visible_type(case_ty);
        let base = super::conversion_labels::comparison_type(&resolved);
        let is_closed_enum = self.resolve_enum_ty(&resolved).is_some()
            || matches!(base, Ty::Option(_) | Ty::Result(_, _));
        let finite = is_closed_enum || matches!(base, Ty::Boolean | Ty::Subrange(_));
        let open = matches!(base, Ty::Integer | Ty::String);
        if !finite && !open {
            return;
        }
        let missing = if coverage_valid {
            self.missing_patterns(rows, case_ty)
        } else {
            Vec::new()
        };
        if has_else && finite {
            let domain = if is_closed_enum {
                "a closed enum".to_string()
            } else {
                format!("finite type `{case_ty}`")
            };
            let (message, hint) = if coverage_valid && missing.is_empty() {
                (
                    format!("`else` is not allowed in a case over {domain}"),
                    if is_closed_enum {
                        "This case already covers every variant. Remove the redundant `else` branch."
                    } else {
                        "This case already covers every value. Remove the redundant `else` branch."
                    },
                )
            } else {
                let message = if missing.is_empty() {
                    format!("`else` is not allowed in a case over {domain}")
                } else {
                    format!(
                        "`else` is not allowed in a case over {domain}: missing {}",
                        missing.join(", ")
                    )
                };
                (message, arms_hint)
            };
            self.error_with_code(SEMA_CASE_ELSE_FORBIDDEN, message, hint, span);
        } else if coverage_valid && open && !has_else {
            let hint = match form {
                CaseForm::Statement => {
                    "Add `else null;` for inputs with no action, or use `if` for selective handling."
                }
                CaseForm::Expression => {
                    "Add `else Value;` before `end case` so every input produces a value."
                }
            };
            self.error_with_code(
                SEMA_NON_EXHAUSTIVE_CASE,
                format!("A `{noun}` over `{case_ty}` needs an `else` arm"),
                hint,
                span,
            );
        } else if coverage_valid && !missing.is_empty() {
            self.error_with_code(
                SEMA_NON_EXHAUSTIVE_CASE,
                format!("Non-exhaustive {noun}: missing {}", missing.join(", ")),
                arms_hint,
                span,
            );
        }
    }
}
