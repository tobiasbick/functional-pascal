//! Shared checking for retained and detached task calls.
//!
//! **Documentation:** `docs/pascal/language/concurrency/go.md`.

use super::{Checker, calls};
use crate::TransferCapability;
use crate::check::MethodCallTarget;
use crate::types::Ty;
use fpas_parser::{DesignatorPart, Expr, PostfixOperation};

impl Checker {
    /// Checks a retained or detached task call, including its escape restrictions.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/go.md`.
    pub(in crate::check) fn check_go_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let previous_error_count = self.errors.len();
        self.check_go_scope_ownership(span);
        self.task_call_depth += 1;
        let inner_ty = match inner {
            Expr::Call {
                designator,
                args,
                span: call_span,
            } => {
                use calls::CallResolution;
                match self.resolve_call_target(inner, designator, args, *call_span, true, None) {
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
        self.task_call_depth -= 1;

        if self.errors.len() == previous_error_count && crate::types::needs_expected_type(&inner_ty)
        {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                "Cannot determine the spawned function's result type",
                "First bind unresolved arguments to explicit types, or use a callable with an explicit result type.",
                span,
            );
        }

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
                self.check_task_transfer_arguments(args);
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
                    self.check_task_transfer_arguments(args);
                }
            }
            _ => {}
        }
        let inner_key = Self::expr_lookup_key(inner);
        self.expr_types.insert(inner_key, inner_ty.clone());
        let result_capability = self.call_result_capability(inner);
        self.set_expr_capability(inner_key, result_capability);
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
        if result_capability.is_task_bound() {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE,
                "Cannot spawn a routine with a task-bound result",
                "Return an immutable value graph from the child, or call this routine on the same task.",
                span,
            );
        }
        Ty::Task(Box::new(inner_ty))
    }

    fn check_task_transfer_arguments(&mut self, args: &[Expr]) {
        for argument in args {
            let value = argument.argument_value();
            if self.expr_is_task_bound(Self::expr_lookup_key(value)) {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE,
                    "Cannot pass a task-bound value to another task",
                    "Pass immutable values without mutable captures, or invoke the routine on the same task.",
                    value.span(),
                );
            } else {
                self.defer_capture_transfer(value, value.span());
            }
        }
    }

    /// Reports whether the resolved call target owns task-bound mutable captures.
    pub(crate) fn callable_expr_is_task_bound(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { designator, .. } => {
                let key = Self::expr_lookup_key(expr);
                if let Some(MethodCallTarget::Static(name)) = self.method_calls.get(&key) {
                    return self.named_callable_capability(name).is_task_bound();
                }
                let target = self
                    .method_calls
                    .get(&key)
                    .map(|target| target.qualified_name())
                    .or_else(|| {
                        self.fluent_calls
                            .get(&key)
                            .map(|target| target.name.as_str())
                    });
                let capability = self.designator_capability(designator);
                let receiver_ty = self
                    .fluent_calls
                    .get(&key)
                    .map(|target| &target.receiver_ty)
                    .or_else(|| {
                        target
                            .and_then(|_| designator.parts.iter().rev().nth(1))
                            .and_then(|part| {
                                let span = match part {
                                    DesignatorPart::Ident(_, span)
                                    | DesignatorPart::Index(_, span) => span,
                                };
                                self.path_types.get(&(span.source_id, span.offset))
                            })
                    });
                let capability =
                    if receiver_ty.is_some_and(|ty| !self.type_can_contain_callable(ty)) {
                        TransferCapability::Transferable
                    } else {
                        capability
                    };
                capability.is_task_bound()
                    || target
                        .is_some_and(|name| self.named_callable_capability(name).is_task_bound())
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                let Some((last, prefix)) = operations.split_last() else {
                    return self.expr_is_task_bound(Self::expr_lookup_key(base));
                };
                let key = Self::postfix_operation_lookup_key(last);
                if let Some(MethodCallTarget::Static(name)) = self.method_calls.get(&key) {
                    return self.named_callable_capability(name).is_task_bound();
                }
                self.postfix_prefix_capability(base, prefix).is_task_bound()
                    || self.fluent_calls.get(&key).is_some_and(|target| {
                        self.named_callable_capability(&target.name).is_task_bound()
                    })
            }
            other => self.expr_is_task_bound(Self::expr_lookup_key(other)),
        }
    }

    fn named_callable_capability(&self, name: &str) -> TransferCapability {
        self.scopes
            .lookup(name)
            .map(|symbol| symbol.capability)
            .unwrap_or_default()
    }
}
