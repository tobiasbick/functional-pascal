//! Statement checking for `if`, `case`, and `panic`.
//!
//! **Documentation:** `docs/pascal/language/control-flow/README.md`, `docs/pascal/language/pattern-matching/README.md`, `docs/pascal/language/error-handling/README.md` (from the repository root).

mod bindings;
mod guards;

use super::super::super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_INVALID_PANIC_ARGUMENT, SEMA_NON_BOOLEAN_CONDITION, SEMA_TYPE_MISMATCH,
};
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
        elsif_branches: &[(Expr, Box<Stmt>)],
        else_branch: Option<&Stmt>,
        span: Span,
    ) {
        self.check_if_condition(condition, span);
        self.check_if_branch(then_branch);
        for (condition, body) in elsif_branches {
            self.check_if_condition(condition, condition.span());
            self.check_if_branch(body);
        }
        if let Some(else_branch) = else_branch {
            self.check_if_branch(else_branch);
        }
    }

    fn check_if_branch(&mut self, branch: &Stmt) {
        self.scopes.push_scope();
        self.check_stmt(branch);
        self.scopes.pop_scope();
    }

    pub(in crate::check) fn check_if_condition(&mut self, condition: &Expr, span: Span) {
        let condition_ty = self.check_expr(condition);
        if matches!(condition_ty, Ty::GenericParam(..)) {
            self.check_type_compat(&Ty::Boolean, &condition_ty, "if condition", span);
        } else if !Ty::Boolean.assignment_compatible_with(&condition_ty) {
            self.error_with_code(
                SEMA_NON_BOOLEAN_CONDITION,
                "Condition must be a boolean expression",
                "if <boolean> then ...",
                span,
            );
        }
    }

    pub(in super::super) fn check_case_stmt(
        &mut self,
        expr: &Expr,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        span: Span,
    ) {
        let case_ty = self.check_expr(expr);
        self.check_case_expression_type(&case_ty, span);

        for arm in arms {
            let binding_sets = arm
                .labels
                .iter()
                .map(|pattern| self.check_pattern(pattern, &case_ty))
                .collect();
            let bindings = self.shared_case_arm_bindings(binding_sets, arm.span);

            self.scopes.push_scope();
            if !bindings.is_empty() {
                for (name, ty) in &bindings {
                    self.scopes.define_with_declaration(
                        name,
                        Symbol {
                            ty: ty.clone(),
                            mutable: false,
                            kind: SymbolKind::Var,
                            task_bound: self.expr_is_task_bound(Self::expr_lookup_key(expr))
                                && self.type_can_contain_callable(ty),
                        },
                        arm.labels
                            .first()
                            .and_then(|pattern| pattern.binding(name).map(|binding| binding.span()))
                            .unwrap_or(arm.span),
                    );
                }
            }
            self.check_guard(&arm.guard, span);
            self.check_stmt(&arm.body);
            self.scopes.pop_scope();
        }

        if let Some(else_body) = else_body {
            self.scopes.push_scope();
            for stmt in else_body {
                self.check_stmt(stmt);
            }
            self.scopes.pop_scope();
        }

        if self.check_recursive_case_coverage(&case_ty, arms, else_body.is_some(), false, span) {
            self.exhaustive_cases.insert(Self::expr_lookup_key(expr));
        }
    }

    /// Require an ordinal, string, or closed variant type as the case scrutinee.
    pub(in crate::check) fn check_case_expression_type(&mut self, case_ty: &Ty, span: Span) {
        let case_ty = self.resolve_visible_type(case_ty);
        if matches!(case_ty, Ty::Result(..) | Ty::Option(_) | Ty::Enum(_))
            || case_ty.is_ordinal()
            || case_ty.compatible_with(&Ty::String)
            || case_ty.is_error()
        {
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
