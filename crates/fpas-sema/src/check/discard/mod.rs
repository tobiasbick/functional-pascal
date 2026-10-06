//! Explicit discard checking and static task-freedom proofs.
//!
//! **Documentation:** `docs/pascal/language/functions/discard.md`

mod expressions;
mod types;
mod unused_results;

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_DISCARD_REQUIRES_VALUE, SEMA_UNSAFE_DISCARD};
use fpas_lexer::Span;
use fpas_parser::Expr;
use types::TaskSafety;

impl Checker {
    /// Completes capture proofs using declared types while preserving callable metadata.
    pub(crate) fn complete_capture_discard_info(
        &self,
        captures: &mut [crate::check::CaptureBinding],
    ) {
        for capture in captures {
            capture.task_free = match self.task_safety(&capture.ty) {
                TaskSafety::Safe => true,
                TaskSafety::Captures => capture.task_free,
                TaskSafety::Forbidden(_) => false,
            };
        }
    }

    /// Requires a value and proves that discarding it cannot lose task handles.
    pub(crate) fn check_discard(&mut self, expr: &Expr, span: Span) {
        let mut operand = expr;
        while let Expr::Paren(inner, _) = operand {
            operand = inner;
        }
        if let Expr::Call {
            designator,
            args,
            span: call_span,
        } = operand
        {
            let name = Self::resolve_designator_name(designator);
            if self
                .scopes
                .lookup(&name)
                .is_some_and(|symbol| matches!(symbol.ty, Ty::Procedure(_)))
            {
                self.check_call_stmt(designator, args, *call_span);
                self.error_with_code(
                    SEMA_DISCARD_REQUIRES_VALUE,
                    "`discard` requires a value; a procedure call has no result",
                    "Call the procedure directly without `discard`.",
                    span,
                );
                return;
            }
        }
        let ty = self.check_expr(expr);
        if ty.is_error() {
            return;
        }
        if matches!(ty, Ty::Unit) {
            self.error_with_code(
                SEMA_DISCARD_REQUIRES_VALUE,
                "`discard` requires a value; a procedure call has no result",
                "Call the procedure directly without `discard`.",
                span,
            );
            return;
        }
        if self.discard_info(expr).value {
            return;
        }
        let reason = match self.task_safety(&ty) {
            TaskSafety::Forbidden(reason) => reason,
            _ => "callable captures are not statically proven free of task handles".into(),
        };
        let hint = if matches!(operand, Expr::Go(_, _)) {
            "Use the statement `go Worker();` instead of `discard go Worker();`."
        } else {
            "Retain and consume task handles. For callables, provide statically known task-free captures; for generic operands use constraints that exclude tasks."
        };
        self.error_with_code(
            SEMA_UNSAFE_DISCARD,
            format!("Cannot discard `{ty}`: {reason}"),
            hint,
            span,
        );
    }

    /// Retains initializer proofs only when storage cannot later hide new captures.
    pub(crate) fn record_binding_discard_info(
        &mut self,
        name: &str,
        ty: &Ty,
        mutable: bool,
        value: Option<&Expr>,
    ) {
        let mut info = value
            .map(|expr| self.discard_info(expr))
            .unwrap_or_default();
        info.value = match self.task_safety(ty) {
            TaskSafety::Safe => true,
            TaskSafety::Captures => {
                !mutable && !matches!(self.resolve_visible_type(ty), Ty::Channel(_)) && info.value
            }
            TaskSafety::Forbidden(_) => false,
        };
        if mutable {
            info.result = false;
        }
        self.scopes.set_discard_info(name, info);
    }

    /// Intersects guarantees from every return in the active routine.
    pub(crate) fn record_discard_return(&mut self, expr: &Expr) {
        let free = self.discard_info(expr).value;
        if let Some(result) = self.discard_results.last_mut() {
            *result &= free;
        }
    }
}
