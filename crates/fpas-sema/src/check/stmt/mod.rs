mod calls;
mod control_flow;

use super::Checker;
use crate::scope::SymbolKind;
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;
use fpas_lexer::Span;
use fpas_parser::*;

impl Checker {
    pub(crate) fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts, _) => {
                self.scopes.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
                self.scopes.pop_scope();
            }

            Stmt::Var(var_def) => self.check_var_def(var_def, false),
            Stmt::MutableVar(var_def) => self.check_var_def(var_def, true),

            Stmt::Assign {
                target,
                value,
                span,
            } => self.check_assign_stmt(target, value, *span),

            Stmt::Return(expr, span) => self.check_return_stmt(expr.as_ref(), *span),
            Stmt::Panic(expr, _) => self.check_panic_stmt(expr),

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => self.check_if_stmt(condition, then_branch, else_branch.as_deref(), *span),

            Stmt::Case {
                expr,
                arms,
                else_body,
                span,
            } => self.check_case_stmt(expr, arms, else_body.as_deref(), *span),

            Stmt::For {
                var_name,
                var_type,
                start,
                direction: _,
                end,
                body,
                span,
            } => self.check_for_stmt(var_name, var_type, start, end, body, *span),

            Stmt::ForIn {
                var_name,
                var_type,
                iterable,
                body,
                span,
            } => self.check_for_in_stmt(var_name, var_type, iterable, body, *span),

            Stmt::While {
                condition,
                body,
                span,
            } => self.check_while_stmt(condition, body, *span),

            Stmt::Repeat {
                body,
                condition,
                span,
            } => self.check_repeat_stmt(body, condition, *span),

            Stmt::Break(span) => self.check_break_stmt(*span),
            Stmt::Continue(span) => self.check_continue_stmt(*span),

            Stmt::Call {
                designator,
                args,
                span,
            } => self.check_call_stmt(designator, args, *span),

            Stmt::Go { expr, span: _ } => {
                // `go` accepts both procedure and function calls.
                if let Expr::Call {
                    designator,
                    args,
                    span: call_span,
                } = expr
                {
                    self.check_call_stmt(designator, args, *call_span);
                } else {
                    self.check_expr(expr);
                }
            }

            Stmt::Select {
                arms, default_body, ..
            } => self.check_select_stmt(arms, default_body.as_deref()),
        }
    }

    fn check_select_stmt(&mut self, arms: &[SelectArm], default_body: Option<&[Stmt]>) {
        for arm in arms {
            let channel_ty = self.check_expr(&arm.channel);
            let declared_ty = self.resolve_type_expr(&arm.type_expr);

            let binding_ty = match channel_ty {
                crate::types::Ty::Channel(inner) => {
                    if !declared_ty.is_error() && !inner.compatible_with(&declared_ty) {
                        self.check_type_compat(
                            &inner,
                            &declared_ty,
                            "select arm binding",
                            arm.span,
                        );
                    }
                    *inner
                }
                crate::types::Ty::Error => crate::types::Ty::Error,
                other => {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!(
                            "Type mismatch in select arm source: expected `Channel(Error)`, found `{other:?}`"
                        ),
                        "Use `case Name: Type from SomeChannel:` with a channel expression.",
                        crate::check::spans::expr_span(&arm.channel),
                    );
                    crate::types::Ty::Error
                }
            };

            self.scopes.push_scope();
            self.scopes.define(
                &arm.binding,
                crate::scope::Symbol {
                    ty: binding_ty,
                    mutable: false,
                    kind: crate::scope::SymbolKind::Var,
                },
            );
            self.check_stmt(&arm.body);
            self.scopes.pop_scope();
        }

        if let Some(body) = default_body {
            for stmt in body {
                self.check_stmt(stmt);
            }
        }
    }

    fn check_assign_stmt(&mut self, target: &Designator, value: &Expr, span: Span) {
        let target_ty = self.check_designator_expr(target);
        let value_ty = self.check_expr(value);

        if !target_ty.is_error() {
            self.check_type_compat(&target_ty, &value_ty, "assignment", span);
        }

        let base_resolved = match target.parts.first() {
            Some(DesignatorPart::Ident(base, _)) => self.scopes.lookup(base).is_some(),
            _ => false,
        };

        if base_resolved && !self.designator_is_mutable_target(target) {
            let target_name = Self::resolve_designator_name(target);
            let hint = match target.parts.first() {
                Some(DesignatorPart::Ident(base, _)) => self
                    .scopes
                    .lookup(base)
                    .map(|symbol| match symbol.kind {
                        SymbolKind::Const => "Constants cannot be reassigned.",
                        SymbolKind::ForVar => "Loop variables are immutable inside the loop body.",
                        SymbolKind::Param => "Mark the parameter `mutable` to allow reassignment.",
                        _ => "Declare with `mutable var` to allow reassignment.",
                    })
                    .unwrap_or("Declare with `mutable var` to allow reassignment."),
                _ => "Declare with `mutable var` to allow reassignment.",
            };

            self.error_with_code(
                SEMA_IMMUTABLE_ASSIGNMENT,
                format!("Cannot assign to `{target_name}`"),
                hint,
                span,
            );
        }
    }
}
