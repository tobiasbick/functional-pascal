//! Shared checking for retained and detached task calls.
//!
//! **Documentation:** `docs/pascal/language/concurrency/go.md`.

use super::{Checker, calls};
use crate::types::Ty;
use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};

impl Checker {
    /// Checks a retained or detached task call, including its escape restrictions.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/go.md`.
    pub(in crate::check) fn check_go_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let inner_ty = match inner {
            Expr::Call {
                designator,
                args,
                span: call_span,
            } => {
                use calls::CallResolution;
                match self.resolve_call_target(inner, designator, args, *call_span, true) {
                    CallResolution::Symbol { kind, ty } => {
                        let name = Self::resolve_designator_name(designator);
                        let dispatch = self.builtin_std_dispatch_name(&name);
                        if dispatch.starts_with("Std.") {
                            self.intrinsic_calls
                                .insert(Self::expr_lookup_key(inner), dispatch);
                        }
                        self.check_known_go_call_symbol(&name, kind, ty, args, *call_span)
                    }
                    CallResolution::MethodResult(ty) => ty,
                    CallResolution::Failed => Ty::Error,
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => self.check_postfix_go(base, operations, span),
            _ => {
                let _ = self.check_expr(inner);
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    "`go` requires a function or procedure call",
                    "Use `go FunctionName(args)` or `go SomeCallable(args)`.",
                    span,
                );
                Ty::Error
            }
        };

        match inner {
            Expr::Call {
                designator, args, ..
            } => {
                self.reject_var_references_in_go(
                    Self::expr_lookup_key(inner),
                    Some(designator),
                    args,
                    span,
                );
            }
            Expr::Postfix { operations, .. } => {
                if let Some(operation @ PostfixOperation::MethodCall { args, .. }) =
                    operations.last()
                {
                    self.reject_var_references_in_go(
                        Self::postfix_operation_lookup_key(operation),
                        None,
                        args,
                        span,
                    );
                }
            }
            _ => {}
        }
        let inner_key = Self::expr_lookup_key(inner);
        self.expr_types.insert(inner_key, inner_ty.clone());
        if self.callable_expr_is_task_bound(inner) {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE,
                "Cannot spawn a task-bound callable across a task boundary",
                "Mutable captures make a closure task-bound. Pass immutable values instead, or invoke the closure on the same task.",
                span,
            );
        } else {
            self.defer_capture_task_call(inner, span);
        }
        Ty::Task(Box::new(inner_ty))
    }

    /// Reports whether the resolved call target owns task-bound mutable captures.
    pub(crate) fn callable_expr_is_task_bound(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { designator, .. } => {
                self.fluent_calls
                    .get(&Self::expr_lookup_key(expr))
                    .and_then(|target| self.scopes.lookup(&target.name))
                    .is_some_and(|symbol| symbol.task_bound)
                    || self.designator_refers_to_task_bound(designator)
            }
            Expr::Postfix { operations, .. } => {
                operations
                    .last()
                    .and_then(|operation| {
                        self.fluent_calls
                            .get(&Self::postfix_operation_lookup_key(operation))
                    })
                    .and_then(|target| self.scopes.lookup(&target.name))
                    .is_some_and(|symbol| symbol.task_bound)
                    || self.expr_is_task_bound(Self::expr_lookup_key(expr))
            }
            other => self.expr_is_task_bound(Self::expr_lookup_key(other)),
        }
    }

    /// Reports whether a named callable or its receiver owns mutable captures.
    pub(crate) fn designator_refers_to_task_bound(&self, designator: &Designator) -> bool {
        let name = Self::resolve_designator_name(designator);
        let Some(symbol) = self.scopes.lookup(&name).or_else(|| {
            designator.parts.first().and_then(|part| match part {
                DesignatorPart::Ident(base, _) => self.scopes.lookup(base),
                _ => None,
            })
        }) else {
            return false;
        };
        if symbol.task_bound {
            return true;
        }
        false
    }
}
