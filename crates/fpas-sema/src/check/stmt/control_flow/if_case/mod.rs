//! Statement checking for `if`, `case`, and `panic`.
//!
//! **Documentation:** `docs/pascal/language/control-flow/README.md`, `docs/pascal/language/pattern-matching/README.md`, `docs/pascal/language/error-handling/README.md` (from the repository root).

mod bindings;
mod coverage;
mod distinct_labels;
mod exhaustiveness;
mod labels;
mod patterns;
mod scalar_bindings;

use super::super::super::Checker;
use crate::types::{EnumTy, Ty};
use coverage::Pat;
use fpas_diagnostics::codes::{
    SEMA_INVALID_PANIC_ARGUMENT, SEMA_TYPE_MISMATCH, SEMA_UNREACHABLE_CASE_LABEL,
};
use fpas_lexer::Span;
use fpas_parser::{CaseArm, CaseLabel, Expr, Stmt};

impl Checker {
    pub(in super::super) fn check_panic_stmt(&mut self, expr: &Expr) {
        let ty = self.check_expr(expr);
        if !ty.compatible_with(&Ty::String) {
            self.error_with_code(
                SEMA_INVALID_PANIC_ARGUMENT,
                "panic() argument must be a string",
                "panic('error message')",
                expr.span(),
            );
        }
    }

    pub(in super::super) fn check_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
        span: Span,
    ) {
        // `is` bindings live in the condition and the then-branch only.
        self.scopes.push_scope();
        self.check_branch_condition(condition, "if", span);
        self.check_stmt(then_branch);
        self.scopes.pop_scope();
        if let Some(else_branch) = else_branch {
            self.check_stmt(else_branch);
        }
    }

    /// Checks existing labels and guards, with a separate scope for every case body.
    pub(in super::super) fn check_case_stmt(
        &mut self,
        expr: &Expr,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        span: Span,
    ) {
        let case_ty = self.check_expr(expr);
        let is_result_or_option = matches!(&case_ty, Ty::Result(_, _) | Ty::Option(_) | Ty::Error);
        let is_data_enum = self.resolve_enum_ty(&case_ty).is_some_and(EnumTy::has_data);
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
        let mut coverage_valid = is_pattern_case && !case_ty.is_error();
        for arm in arms {
            if let Some(binding_name) =
                self.scalar_case_binding(is_scalar_case, &arm.labels, &arm.guard)
            {
                self.check_import_alias_collision(binding_name, arm.span);
                self.scopes.push_scope();
                self.define_pattern_binding(binding_name, &case_ty, expr, arm.span);
                self.check_guard(&arm.guard, span);
                self.check_stmt(&arm.body);
                self.scopes.pop_scope();
                continue;
            }

            let mut binding_sets = Vec::with_capacity(arm.labels.len());
            for label in &arm.labels {
                let errors_before = self.errors.len();
                let checked = self.check_case_label(&case_ty, is_pattern_case, label);
                if self.errors.len() != errors_before {
                    coverage_valid = false;
                }
                if coverage_valid {
                    let row = vec![checked.pat];
                    if !self.pattern_row_is_useful(&rows, &row, std::slice::from_ref(&case_ty)) {
                        self.error_with_code(
                            SEMA_UNREACHABLE_CASE_LABEL,
                            "Case label is unreachable; earlier arms already match every value it matches",
                            "Remove the label, or move it before the arm that already covers it.",
                            label_span(label),
                        );
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
            self.check_guard(&arm.guard, span);
            self.check_stmt(&arm.body);
            if !bindings.is_empty() {
                self.scopes.pop_scope();
            }
        }

        if let Some(else_body) = else_body {
            self.scopes.push_scope();
            for stmt in else_body {
                self.check_stmt(stmt);
            }
            self.scopes.pop_scope();
        }

        self.check_case_exhaustiveness(&case_ty, &rows, coverage_valid, else_body.is_some(), span);
    }

    fn check_case_expression_type(
        &mut self,
        case_ty: &Ty,
        is_result_or_option: bool,
        is_data_enum: bool,
        is_simple_enum: bool,
        span: Span,
    ) {
        let selector_ty = distinct_labels::comparison_type(case_ty);
        if is_result_or_option
            || is_data_enum
            || is_simple_enum
            || selector_ty.is_ordinal()
            || selector_ty.compatible_with(&Ty::String)
            || selector_ty.is_error()
        {
            return;
        }
        if let Ty::Distinct(distinct) = case_ty {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Distinct type `{}` cannot be a case selector because its underlying type `{}` is not ordinal or string",
                    distinct.name, distinct.underlying
                ),
                format!(
                    "Compare the value with `if`, for example `if Value < {}(1.0) then`.",
                    distinct.name
                ),
                span,
            );
            return;
        }

        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            "Case expression must be an ordinal, string, Result, or Option type",
            "Use integer, boolean, enum, string, Result, or Option.",
            span,
        );
    }
}

fn label_span(label: &CaseLabel) -> Span {
    match label {
        CaseLabel::Value { span, .. } | CaseLabel::Binding { span, .. } => *span,
        CaseLabel::Pattern(pattern) => pattern.span(),
    }
}
