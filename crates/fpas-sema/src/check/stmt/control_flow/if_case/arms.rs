//! Shared `case` checking for statements and expressions: selector type, labels,
//! arm bindings, guards, unreachable labels, and coverage rows.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md` and
//! `docs/pascal/language/pattern-matching/exhaustiveness.md`

use super::Checker;
use super::coverage::Pat;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_UNREACHABLE_CASE_LABEL;
use fpas_lexer::Span;
use fpas_parser::{CaseLabel, Expr};

/// Labels, guard, and span of one arm, independent of its statement or value body.
pub(super) struct CaseArmHead<'a> {
    pub(super) labels: &'a [CaseLabel],
    pub(super) guard: &'a Option<Expr>,
    pub(super) span: Span,
}

/// Selector type and unguarded label rows collected while checking the arms.
pub(super) struct CaseArms {
    pub(super) case_ty: Ty,
    pub(super) rows: Vec<Vec<Pat>>,
    /// Whether closed-enum coverage applies (enums, `Option`, `Result`) and labels checked cleanly.
    pub(super) pattern_coverage: bool,
    /// Whether every label checked without errors, so coverage diagnostics are meaningful.
    pub(super) labels_valid: bool,
}

impl Checker {
    /// Checks the selector and every arm; `check_body` checks arm `index` inside its binding scope.
    pub(super) fn check_case_arms(
        &mut self,
        expr: &Expr,
        arms: &[CaseArmHead<'_>],
        span: Span,
        mut check_body: impl FnMut(&mut Self, usize),
    ) -> CaseArms {
        let case_ty = self.check_expr(expr);
        let is_result_or_option = matches!(&case_ty, Ty::Result(_, _) | Ty::Option(_) | Ty::Error);
        let is_data_enum = self
            .resolve_enum_ty(&case_ty)
            .is_some_and(|enumeration| enumeration.has_data());
        let is_simple_enum = self
            .resolve_enum_ty(&case_ty)
            .is_some_and(|enum_ty| !enum_ty.has_data());

        self.check_case_expression_type(
            &case_ty,
            is_result_or_option,
            is_data_enum,
            is_simple_enum,
            span,
        );

        let is_scalar_case = !is_result_or_option && !is_data_enum;
        let is_pattern_case = is_result_or_option || is_data_enum || is_simple_enum;
        let mut rows: Vec<Vec<Pat>> = Vec::new();
        let mut labels_valid = !case_ty.is_error();
        for (index, arm) in arms.iter().enumerate() {
            if let Some(binding_name) =
                self.scalar_case_binding(is_scalar_case, arm.labels, arm.guard)
            {
                self.check_import_alias_collision(binding_name, arm.span);
                self.scopes.push_scope();
                self.define_pattern_binding(binding_name, &case_ty, expr, arm.span);
                self.check_guard(arm.guard, span);
                check_body(self, index);
                self.scopes.pop_scope();
                continue;
            }

            let mut binding_sets = Vec::with_capacity(arm.labels.len());
            for label in arm.labels {
                let errors_before = self.errors.len();
                let checked = self.check_case_label(&case_ty, is_pattern_case, label);
                if self.errors.len() != errors_before {
                    labels_valid = false;
                }
                if labels_valid {
                    let row = vec![checked.pat];
                    if !self.pattern_row_is_useful(&rows, &row, std::slice::from_ref(&case_ty)) {
                        // Scalar duplicates keep their existing statement behavior.
                        if is_pattern_case {
                            self.error_with_code(
                                SEMA_UNREACHABLE_CASE_LABEL,
                                "Case label is unreachable; earlier arms already match every value it matches",
                                "Remove the label, or move it before the arm that already covers it.",
                                label_span(label),
                            );
                        }
                    } else if arm.guard.is_none() {
                        rows.push(row);
                    }
                }
                binding_sets.push(checked.bindings);
            }
            let bindings = self.shared_case_arm_bindings(binding_sets, arm.span);

            if !bindings.is_empty() {
                self.scopes.push_scope();
                for (name, ty) in &bindings {
                    self.check_import_alias_collision(name, arm.span);
                    self.define_pattern_binding(name, ty, expr, arm.span);
                }
            }
            self.check_guard(arm.guard, span);
            check_body(self, index);
            if !bindings.is_empty() {
                self.scopes.pop_scope();
            }
        }
        CaseArms {
            case_ty,
            rows,
            pattern_coverage: is_pattern_case && labels_valid,
            labels_valid,
        }
    }
}

fn label_span(label: &CaseLabel) -> Span {
    match label {
        CaseLabel::Value { span, .. } | CaseLabel::Binding { span, .. } => *span,
        CaseLabel::Pattern(pattern) => pattern.span(),
    }
}
