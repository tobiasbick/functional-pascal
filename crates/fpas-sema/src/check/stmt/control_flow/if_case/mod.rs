//! Statement checking for `if`, `case`, and `panic`.
//!
//! **Documentation:** `docs/pascal/03-control-flow.md`, `docs/pascal/06-pattern-matching.md`, `docs/pascal/07-error-handling.md` (from the repository root).

mod exhaustiveness;
mod labels;

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
                super::super::super::spans::expr_span(expr),
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
        let condition_ty = self.check_expr(condition);
        if !condition_ty.compatible_with(&Ty::Boolean) {
            self.error_with_code(
                SEMA_NON_BOOLEAN_CONDITION,
                "Condition must be a boolean expression",
                "if <boolean> then ...",
                span,
            );
        }

        self.check_stmt(then_branch);
        if let Some(else_branch) = else_branch {
            self.check_stmt(else_branch);
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
        let is_result_or_option = matches!(&case_ty, Ty::Result(_, _) | Ty::Option(_) | Ty::Error);
        let is_data_enum = matches!(&case_ty, Ty::Enum(enum_ty) if enum_ty.has_data());
        let is_simple_enum = matches!(&case_ty, Ty::Enum(enum_ty) if !enum_ty.has_data());

        self.check_case_expression_type(
            &case_ty,
            is_result_or_option,
            is_data_enum,
            is_simple_enum,
            span,
        );

        for arm in arms {
            let mut binding_sets = Vec::new();
            for label in &arm.labels {
                if let Some(bindings) =
                    self.check_case_label(&case_ty, is_result_or_option, is_data_enum, label)
                {
                    binding_sets.push(bindings);
                }
            }

            let labels_with_bindings = binding_sets
                .iter()
                .filter(|bindings| !bindings.is_empty())
                .count();
            if labels_with_bindings == 0 {
                self.check_guard(&arm.guard, span);
                self.check_stmt(&arm.body);
                continue;
            }

            for bindings in binding_sets
                .into_iter()
                .filter(|bindings| !bindings.is_empty())
            {
                self.scopes.push_scope();
                for (name, ty) in &bindings {
                    self.scopes.define(
                        name,
                        Symbol {
                            ty: ty.clone(),
                            mutable: false,
                            kind: SymbolKind::Var,
                        },
                    );
                }
                self.check_guard(&arm.guard, span);
                self.check_stmt(&arm.body);
                self.scopes.pop_scope();
            }
        }

        if let Some(else_body) = else_body {
            for stmt in else_body {
                self.check_stmt(stmt);
            }
        }

        if else_body.is_none() {
            self.check_exhaustiveness(&case_ty, arms, span);
        }
    }

    fn check_case_expression_type(
        &mut self,
        case_ty: &Ty,
        is_result_or_option: bool,
        is_data_enum: bool,
        is_simple_enum: bool,
        span: Span,
    ) {
        if is_result_or_option
            || is_data_enum
            || is_simple_enum
            || case_ty.is_ordinal()
            || case_ty.compatible_with(&Ty::String)
            || case_ty.is_error()
        {
            return;
        }

        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            "Case expression must be an ordinal, string, Result, or Option type",
            "Use integer, boolean, char, enum, string, Result, or Option.",
            span,
        );
    }
}
