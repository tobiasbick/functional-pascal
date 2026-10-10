//! Lifetime rules: a `var` parameter must not outlive the call that supplied it.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use super::super::Checker;
use super::super::closures::CaptureBinding;
use fpas_diagnostics::codes::SEMA_VAR_PARAMETER_ESCAPE;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Rejects an anonymous closure that captures a `var` parameter.
    pub(in crate::check) fn reject_closure_var_captures(
        &mut self,
        captures: &[CaptureBinding],
        span: Span,
    ) {
        if let Some(capture) = captures.iter().find(|capture| capture.reference) {
            self.error_with_code(
                SEMA_VAR_PARAMETER_ESCAPE,
                format!("A closure cannot capture `var` parameter `{}`", capture.name),
                format!(
                    "The closure could outlive the call. Copy the value into a local first, for example `const Current: {} := {};`, and capture the copy.",
                    capture.ty, capture.name
                ),
                span,
            );
        }
    }

    /// Remembers whether a named nested routine uses an enclosing `var` parameter.
    pub(in crate::check) fn record_var_parameter_routine(
        &mut self,
        key: usize,
        parameter: Option<String>,
    ) {
        match parameter {
            Some(parameter) => {
                self.var_parameter_routines.insert(key, parameter.clone());
                for usage in self
                    .pending_var_parameter_uses
                    .remove(&key)
                    .unwrap_or_default()
                {
                    self.report_pending_routine_use(&usage, &parameter);
                }
            }
            None => {
                self.var_parameter_routines.remove(&key);
                for usage in self
                    .pending_var_parameter_uses
                    .remove(&key)
                    .unwrap_or_default()
                {
                    self.check_pending_routine_use(key, usage);
                }
            }
        }
    }

    /// Rejects a nested routine used as a value while it uses an enclosing `var` parameter.
    pub(in crate::check) fn reject_var_parameter_routine_value(&mut self, designator: &Designator) {
        let [DesignatorPart::Ident(name, span)] = designator.parts.as_slice() else {
            return;
        };
        if let Some(parameter) = self.var_parameter_routine(name) {
            self.report_routine_reference_escape(name, &parameter, *span, false);
        } else {
            self.defer_routine_reference_use(name, *span, false);
        }
    }

    /// Rejects explicit/implicit writable arguments and `var`-parameter routines in `go`.
    pub(in crate::check) fn reject_var_references_in_go(
        &mut self,
        call_key: usize,
        callee: Option<&Designator>,
        args: &[Expr],
        span: Span,
    ) {
        if let Some(arg) = args
            .iter()
            .find(|arg| matches!(arg.argument_value(), Expr::VarArgument { .. }))
        {
            self.error_with_code(
                SEMA_VAR_PARAMETER_ESCAPE,
                "`go` cannot pass a `var` argument",
                "The task could outlive the caller's variable. Pass a value and return the result through the task, for example `const T: task := go Compute(Value);`.",
                arg.span(),
            );
            return;
        }
        let intrinsic = self
            .fluent_calls
            .get(&call_key)
            .map(|target| target.name.as_str())
            .or_else(|| self.intrinsic_calls.get(&call_key).map(String::as_str));
        if intrinsic.is_some_and(|name| {
            crate::std_registry::intrinsic_std_receiver_mode(name) == crate::types::ParamMode::Var
        }) {
            self.error_with_code(
                SEMA_VAR_PARAMETER_ESCAPE,
                "`go` cannot pass an implicit writable receiver",
                "Call `Items.Push(Value)` or `Items.Pop()` on the current task. A writable receiver follows the same lifetime rules as an explicit `var` argument.",
                span,
            );
            return;
        }
        if let Some([DesignatorPart::Ident(name, _)]) =
            callee.map(|designator| designator.parts.as_slice())
        {
            if let Some(parameter) = self.var_parameter_routine(name) {
                self.report_routine_reference_escape(name, &parameter, span, true);
            } else {
                self.defer_routine_reference_use(name, span, true);
            }
        }
    }

    /// Reports a known reference capture at a routine-value or task use.
    pub(super) fn report_routine_reference_escape(
        &mut self,
        name: &str,
        parameter: &str,
        span: Span,
        task: bool,
    ) {
        let (message, hint) = if task {
            (
                format!("`go` cannot start `{name}` because it uses `var` parameter `{parameter}`"),
                format!("Call `{name}` directly, or pass `{parameter}` to the task by value."),
            )
        } else {
            (
                format!(
                    "`{name}` uses `var` parameter `{parameter}` and cannot be used as a value"
                ),
                format!(
                    "Call `{name}` directly while the enclosing routine runs, or pass `{parameter}` to it as a parameter."
                ),
            )
        };
        self.error_with_code(SEMA_VAR_PARAMETER_ESCAPE, message, hint, span);
    }
}
