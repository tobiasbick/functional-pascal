//! `case` expressions: the shared arm checks, one arm-value type, and complete coverage.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`

use super::Checker;
use super::arms::{CaseArmHead, CaseArms};
use super::coverage::Pat;
use super::exhaustiveness::CaseForm;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE;
use fpas_lexer::Span;
use fpas_parser::{CaseExprArm, CaseExprElse, Expr};

impl Checker {
    /// Checks every arm and returns the shared type of the arm values.
    pub(in crate::check) fn check_case_expr(
        &mut self,
        selector: &Expr,
        arms: &[CaseExprArm],
        else_arm: Option<&CaseExprElse>,
        span: Span,
        expected: Option<&Ty>,
    ) -> Ty {
        let heads = arms
            .iter()
            .map(|arm| CaseArmHead {
                labels: &arm.labels,
                guard: &arm.guard,
                span: arm.span,
            })
            .collect::<Vec<_>>();
        let mut values = Vec::with_capacity(arms.len() + 1);
        let checked = self.check_case_arms(selector, &heads, span, |checker, index| {
            let value = &arms[index].value;
            values.push((
                checker.check_expr_with_expected(value, expected),
                value.span(),
            ));
        });
        if let Some(else_arm) = else_arm {
            values.push((
                self.check_expr_with_expected(&else_arm.value, expected),
                else_arm.value.span(),
            ));
        }
        self.check_case_expression_coverage(&checked, else_arm.is_some(), span);
        self.shared_branch_type(values, "`case` expression arms")
    }

    /// Every selector value must produce a value: closed types need every variant,
    /// and open domains need an `else` arm.
    fn check_case_expression_coverage(&mut self, checked: &CaseArms, has_else: bool, span: Span) {
        self.check_case_exhaustiveness(
            &checked.case_ty,
            &checked.rows,
            checked.pattern_coverage,
            has_else,
            CaseForm::Expression,
            span,
        );
        if has_else
            || checked.pattern_coverage
            || !checked.labels_valid
            || !self.pattern_row_is_useful(
                &checked.rows,
                &[Pat::Wild],
                std::slice::from_ref(&checked.case_ty),
            )
        {
            return;
        }
        let missing = self.missing_patterns(&checked.rows, &checked.case_ty);
        if missing.is_empty() {
            self.error_with_code(
                SEMA_NON_EXHAUSTIVE_CASE,
                format!(
                    "A `case` expression over `{}` needs an `else` arm",
                    checked.case_ty
                ),
                "The arms cannot list every value of this type. Add `else Value;` before `end case` so every input produces a value.",
                span,
            );
        } else {
            self.error_with_code(
                SEMA_NON_EXHAUSTIVE_CASE,
                format!(
                    "Non-exhaustive case expression: missing {}",
                    missing.join(", ")
                ),
                "Add a `when` arm with a value for each missing value. Guarded arms do not count toward coverage.",
                span,
            );
        }
    }
}
