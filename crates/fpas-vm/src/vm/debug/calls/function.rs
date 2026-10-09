//! Typed detached worker execution. See `docs/pascal/tools/debugger.md`.

use super::detach::error;
use super::execute::{CallSandbox, runtime_error};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::dispatch::DispatchStep;
use crate::vm::hosted::HostedState;
use crate::vm::worker::Worker;
use fpas_bytecode::{DebugEffectSet, FunctionId, Value};
use std::sync::Arc;

impl CallSandbox {
    /// Invokes a typed, effect-checked detached worker. See `docs/pascal/tools/debugger.md`.
    pub(super) fn invoke_function(
        &mut self,
        function: FunctionId,
        bound_receiver: Option<&Value>,
        captures: &[Value],
        mut arguments: Vec<Value>,
        display_name: &str,
    ) -> Result<Value, DebugSessionError> {
        let info = self
            .executable
            .executable()
            .functions
            .get(usize::from(function.get()))
            .ok_or_else(|| {
                error(
                    DebugErrorKind::UnknownCallable,
                    format!("debug callable `{display_name}` references a missing function"),
                    "Rebuild the executable with the current compiler.",
                )
            })?;
        let visible_arity = usize::from(info.arity)
            .checked_sub(usize::from(bound_receiver.is_some()))
            .ok_or_else(|| {
                error(
                    DebugErrorKind::CallArity,
                    format!("debug callable `{display_name}` has no receiver parameter"),
                    "Rebuild the executable with current bound-method metadata.",
                )
            })?;
        if visible_arity != arguments.len() {
            return Err(error(
                DebugErrorKind::CallArity,
                format!(
                    "debug callable `{display_name}` expects {} arguments, received {}",
                    visible_arity,
                    arguments.len()
                ),
                "Pass the exact declared argument count.",
            ));
        }
        if usize::from(info.capture_count) != captures.len() {
            return Err(error(
                DebugErrorKind::CallArity,
                format!(
                    "debug callable `{display_name}` expects {} captures, received {}",
                    info.capture_count,
                    captures.len()
                ),
                "Invoke nested routines through their visible first-class function value.",
            ));
        }
        let effects = self
            .effects
            .get(usize::from(function.get()))
            .copied()
            .unwrap_or(DebugEffectSet::UNKNOWN);
        self.require_safe(display_name, effects)?;
        if let Some(receiver) = bound_receiver {
            arguments.insert(0, receiver.clone());
        }
        self.require_parameters(info, &arguments, display_name)?;
        let arguments = self.detach_values(&arguments)?;
        let captures = self.detach_values(captures)?;
        let mut worker = Worker::for_function_with_captures(
            Arc::clone(&self.executable),
            function,
            &arguments,
            &captures,
            Arc::clone(&self.globals),
            Arc::clone(&self.layouts),
            Arc::new(HostedState::new(fpas_std::Console::new(), Vec::new())),
        )
        .map_err(|diagnostic| runtime_error(*diagnostic))?;
        loop {
            self.check_running()?;
            if self.instructions >= self.limits.max_call_instructions {
                return Err(error(
                    DebugErrorKind::CallLimit,
                    format!(
                        "debug call instruction count exceeds limit {}",
                        self.limits.max_call_instructions
                    ),
                    "Use a smaller bounded callable.",
                ));
            }
            match worker
                .dispatch_one()
                .map_err(|diagnostic| runtime_error(*diagnostic))?
            {
                DispatchStep::Continue => {}
                DispatchStep::Return(value) => return Ok(value),
                DispatchStep::Suspend => {
                    return Err(error(
                        DebugErrorKind::ForbiddenCallEffect,
                        "debug call attempted to suspend on task scheduling",
                        "Remove task and scheduler operations from debugger-call targets.",
                    ));
                }
            }
            self.instructions = self.instructions.saturating_add(1);
            if worker.call_stack.len() > self.limits.max_call_depth {
                return Err(error(
                    DebugErrorKind::CallLimit,
                    format!(
                        "debug call depth exceeds limit {}",
                        self.limits.max_call_depth
                    ),
                    "Use a shallower call chain or recursion depth.",
                ));
            }
        }
    }

    /// Copies value arguments while retaining references owned by this sandbox.
    /// See `docs/pascal/tools/debugger.md`.
    pub(super) fn detach_values(
        &mut self,
        values: &[Value],
    ) -> Result<Vec<Value>, DebugSessionError> {
        values
            .iter()
            .map(|value| self.detach_argument(value))
            .collect()
    }
}
