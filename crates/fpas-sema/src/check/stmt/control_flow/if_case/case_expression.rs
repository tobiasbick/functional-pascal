//! `case` expressions: the shared arm checks, one arm-value type, and complete coverage.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`

use super::Checker;
use super::arms::CaseArmHead;
use super::exhaustiveness::CaseForm;
use crate::types::Ty;
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
        let previous_errors = self.errors.len();
        self.deferred_constructor_inference += 1;
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
            true
        });
        if let Some(else_arm) = else_arm {
            values.push((
                self.check_expr_with_expected(&else_arm.value, expected),
                else_arm.value.span(),
            ));
        }
        let mut exits = checked.exits;
        if else_arm.is_some() {
            exits.push(self.scopes.snapshot_capabilities());
        }
        self.scopes.join_capability_snapshots(&exits);
        self.deferred_constructor_inference -= 1;
        self.check_case_exhaustiveness(
            &checked.case_ty,
            &checked.rows,
            checked.labels_valid,
            else_arm.is_some(),
            CaseForm::Expression,
            span,
        );
        if self.errors.len() == previous_errors
            && let Some(context) = self.branch_completion_context(&values)
        {
            return self.check_case_expr(selector, arms, else_arm, span, Some(&context));
        }
        self.shared_branch_type(values, "`case` expression arms", expected)
    }
}
