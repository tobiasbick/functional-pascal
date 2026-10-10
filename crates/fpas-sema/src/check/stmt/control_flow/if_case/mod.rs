//! Statement checking for `if`, `case`, and `panic`.
//!
//! **Documentation:** `docs/pascal/language/control-flow/README.md`, `docs/pascal/language/pattern-matching/README.md`, `docs/pascal/language/error-handling/README.md` (from the repository root).

mod arms;
mod bindings;
mod case_expression;
mod coverage;
mod distinct_labels;
mod exhaustiveness;
mod labels;
mod patterns;
mod scalar_bindings;

use super::super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_INVALID_PANIC_ARGUMENT, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::{CaseArm, Expr, Stmt};

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
        let heads = arms
            .iter()
            .map(|arm| arms::CaseArmHead {
                labels: &arm.labels,
                guard: &arm.guard,
                span: arm.span,
            })
            .collect::<Vec<_>>();
        let checked = self.check_case_arms(expr, &heads, span, |checker, index| {
            checker.check_stmt(&arms[index].body);
        });

        if let Some(else_body) = else_body {
            self.scopes.push_scope();
            for stmt in else_body {
                self.check_stmt(stmt);
            }
            self.scopes.pop_scope();
        }

        self.check_case_exhaustiveness(
            &checked.case_ty,
            &checked.rows,
            checked.pattern_coverage,
            else_body.is_some(),
            exhaustiveness::CaseForm::Statement,
            span,
        );
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
