//! Shared instruction-boundary and terminal-opcode transitions.

use super::StepResult;
use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;
use std::sync::atomic::Ordering;

/// Execution policy for the bytecode currently loaded into a worker.
#[derive(Clone, Copy)]
pub(super) enum ExecutionContext {
    /// The program's main task, including an optional image entry procedure.
    Main,
    /// A task created by retained or detached spawn bytecode.
    SpawnedTask,
    /// A callback driven synchronously by a hosted intrinsic.
    SynchronousCallback,
}

impl ExecutionContext {
    fn description(self) -> &'static str {
        match self {
            Self::Main => "main task",
            Self::SpawnedTask => "spawned task",
            Self::SynchronousCallback => "synchronous callback",
        }
    }
}

/// State change produced by one validated instruction step.
pub(super) enum ExecutionTransition {
    /// Continue executing the currently loaded bytecode.
    Continue,
    /// The current main or spawned task returned its final value.
    Completed(Value),
    /// The current spawned task parked in a cooperative wait queue.
    Suspended,
    /// Runtime failure or cooperative shutdown canceled the current execution context.
    Cancelled,
}

impl Worker {
    /// Validate a bytecode entry before changing stacks or instruction pointers.
    pub(super) fn validate_code_entry(
        &self,
        entry: usize,
        location: SourceLocation,
    ) -> Result<(), VmError> {
        let code_len = self.shared.chunk.code().len();
        if entry < code_len {
            return Ok(());
        }

        Err(internal_error(
            format!("Bytecode entry {entry} is out of bounds (code len {code_len})"),
            "This indicates malformed bytecode or a compiler/linker control-flow bug. Please report it.",
            location,
        ))
    }

    /// Execute one instruction and apply the shared boundary, `Halt`, `Return`, and suspension
    /// policy for `context`.
    pub(super) fn advance_execution(
        &mut self,
        context: ExecutionContext,
        fallback_location: SourceLocation,
    ) -> Result<ExecutionTransition, VmError> {
        if let Some(transition) = self.shutdown_transition(context)? {
            return Ok(transition);
        }

        let code_len = self.shared.chunk.code().len();
        if self.ip >= code_len {
            return Err(internal_error(
                format!(
                    "Instruction pointer reached the code boundary without a terminal instruction: ip={}, len={code_len}, context={}",
                    self.ip,
                    context.description()
                ),
                "Main bytecode must end with `Halt`; functions and spawned tasks must end with `Return`. This indicates malformed bytecode or a VM control-flow bug.",
                self.current_location,
            ));
        }

        match self.exec_one(fallback_location)? {
            StepResult::Continue => Ok(ExecutionTransition::Continue),
            StepResult::Suspended => {
                if matches!(context, ExecutionContext::SpawnedTask) {
                    Ok(ExecutionTransition::Suspended)
                } else {
                    Err(internal_error(
                        format!("Task suspended during {} execution", context.description()),
                        "Only spawned bytecode may suspend. This indicates malformed bytecode or a VM dispatch bug.",
                        self.current_location,
                    ))
                }
            }
            StepResult::Halt => self.handle_halt(context),
            StepResult::Return => self.handle_return(context),
        }
    }

    fn shutdown_transition(
        &self,
        context: ExecutionContext,
    ) -> Result<Option<ExecutionTransition>, VmError> {
        match context {
            ExecutionContext::Main if self.shared.is_shutdown() => Err(runtime_error(
                RUNTIME_VM_SHUTDOWN,
                "Execution aborted: a concurrent task failed",
                "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                self.current_location,
            )),
            ExecutionContext::SpawnedTask
                if self.shared.abort_spawned_bytecode.load(Ordering::Acquire) =>
            {
                Ok(Some(ExecutionTransition::Cancelled))
            }
            ExecutionContext::SynchronousCallback
                if self.shared.is_shutdown() && !self.allow_shutdown_during_sync_call =>
            {
                if self.current_task_id == 0 {
                    Err(runtime_error(
                        RUNTIME_VM_SHUTDOWN,
                        "Execution aborted: a concurrent task failed",
                        "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                        self.current_location,
                    ))
                } else {
                    Ok(Some(ExecutionTransition::Cancelled))
                }
            }
            _ => Ok(None),
        }
    }

    fn handle_halt(&mut self, context: ExecutionContext) -> Result<ExecutionTransition, VmError> {
        if !matches!(context, ExecutionContext::Main) || !self.call_stack.is_empty() {
            return Err(internal_error(
                format!("Halt is invalid during {} execution", context.description()),
                "Only frame-free main bytecode may use `Halt`; functions and spawned tasks must return with `Return`.",
                self.current_location,
            ));
        }

        if let Some(entry_ip) = self.pending_entry_ip.take() {
            self.validate_code_entry(entry_ip, self.current_location)?;
            self.ip = entry_ip;
            return Ok(ExecutionTransition::Continue);
        }

        Ok(ExecutionTransition::Completed(Value::Unit))
    }

    fn handle_return(&mut self, context: ExecutionContext) -> Result<ExecutionTransition, VmError> {
        let return_value = self.pop(self.current_location)?;
        if let Some(frame) = self.call_stack.pop() {
            self.stack.truncate(frame.base_slot);
            self.push(return_value)?;
            self.ip = frame.return_ip;
            return Ok(ExecutionTransition::Continue);
        }

        if matches!(context, ExecutionContext::SynchronousCallback) {
            return Err(internal_error(
                "Synchronous callback returned past its injected call frame",
                "This indicates malformed bytecode or a VM callback-stack bug. Please report it.",
                self.current_location,
            ));
        }

        Ok(ExecutionTransition::Completed(return_value))
    }
}
