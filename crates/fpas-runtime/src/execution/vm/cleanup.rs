//! Task-owned lexical resource obligations.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`

use super::worker::Worker;
use crate::{OwnershipError, SharedFunction, TraceRoots, Value};
use fpas_code::CleanupFamily;

/// One scope remains owned by the task while its release procedure runs or suspends.
pub(super) struct CleanupScope {
    pub frame_depth: usize,
    pub initializer_count: usize,
    pub task_scope_depth: usize,
    pub callback_depth: usize,
    pub resources: Vec<CleanupResource>,
}

pub(super) struct CleanupResource {
    pub value: Value,
    pub disposer: SharedFunction,
    pub family: CleanupFamily,
}

pub(super) struct CleanupExit {
    pub frame_depth: usize,
    pub resources: Vec<CleanupResource>,
    pub awaiting_depth: Option<usize>,
    pub recovery_depth: Option<usize>,
    pub scope_base: usize,
    pub callback_base: usize,
    pub family: CleanupFamily,
    pub result: Option<Value>,
    pub current_resource: Option<Value>,
    pub protected: bool,
    pub failed: bool,
}

pub(super) struct ResourceInitializer {
    depth: usize,
    callbacks: usize,
    destination: Option<u16>,
    armed: bool,
    ready: bool,
}

#[derive(Default)]
pub(super) struct CleanupState {
    pub scopes: Vec<CleanupScope>,
    pub initializers: Vec<ResourceInitializer>,
    pub exits: Vec<CleanupExit>,
    pub pending_failure: Option<super::VmError>,
    pub deferred_cancel: Option<super::VmError>,
}

impl TraceRoots for CleanupState {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        for scope in &self.scopes {
            for resource in &scope.resources {
                visit(&resource.value);
                visit(&Value::Function(resource.disposer.clone()));
            }
        }
        for exit in &self.exits {
            for resource in &exit.resources {
                visit(&resource.value);
                visit(&Value::Function(resource.disposer.clone()));
            }
            if let Some(result) = &exit.result {
                visit(result);
            }
            if let Some(resource) = &exit.current_resource {
                visit(resource);
            }
        }
        Ok(())
    }
}

impl Worker {
    pub(super) fn enter_cleanup_scope(&mut self) {
        self.cleanup.scopes.push(CleanupScope {
            frame_depth: self.call_stack.len(),
            initializer_count: self.cleanup.initializers.len(),
            task_scope_depth: self.task_scopes.len(),
            callback_depth: self.callback_continuations.len(),
            resources: Vec::new(),
        });
    }

    pub(super) fn begin_resource_init(&mut self) {
        self.cleanup.initializers.push(ResourceInitializer {
            depth: self.call_stack.len(),
            callbacks: self.callback_continuations.len(),
            destination: None,
            armed: false,
            ready: false,
        });
    }

    pub(super) fn arm_resource_init(&mut self) {
        if let Some(initializer) = self.cleanup.initializers.last_mut() {
            initializer.depth = self.call_stack.len();
            initializer.callbacks = self.callback_continuations.len();
            initializer.destination = None;
            initializer.armed = true;
        }
    }

    pub(super) fn observe_resource_producer(&mut self, instruction: crate::DecodedInstruction) {
        if let Some(initializer) = self.cleanup.initializers.last_mut()
            && initializer.armed
            && initializer.destination.is_none()
            && initializer.depth == self.call_stack.len()
        {
            initializer.destination = Some(instruction.abc().a);
        }
    }

    pub(super) fn complete_resource_producer(&mut self, returned: Option<&Value>) {
        let Some(initializer) = self.cleanup.initializers.last() else {
            return;
        };
        if !initializer.armed
            || self.call_stack.len() != initializer.depth
            || self.task_suspension.is_some()
            || self.callback_continuations.len() > initializer.callbacks
        {
            return;
        }
        let Some(destination) = initializer.destination else {
            return;
        };
        let value = returned.or_else(|| self.read_operand(destination).ok());
        let Some(value) = value else { return };
        let acquired = producer_succeeded(value);
        if let Some(initializer) = self.cleanup.initializers.last_mut() {
            initializer.ready = acquired;
            initializer.armed = false;
        }
    }

    pub(super) fn resource_handoff_pending(&self) -> bool {
        self.cleanup
            .initializers
            .last()
            .is_some_and(|initializer| initializer.ready)
    }

    pub(super) fn cleanup_masks_cancellation(&self) -> bool {
        !self.cleanup.exits.is_empty()
            || self.cleanup.pending_failure.is_some()
            || self.resource_handoff_pending()
    }

    pub(in crate::execution::vm) fn has_cleanup_obligations(&self) -> bool {
        !self.cleanup.exits.is_empty()
            || self.resource_handoff_pending()
            || matches!(
                self.task_suspension,
                Some(
                    super::tasks::TaskSuspension::Acp(_)
                        | super::tasks::TaskSuspension::Mcp(_)
                        | super::tasks::TaskSuspension::NativeRetirement { .. }
                )
            )
            || self
                .cleanup
                .scopes
                .iter()
                .any(|scope| !scope.resources.is_empty())
    }

    pub(in crate::execution::vm) fn defer_cleanup_cancel(&mut self, error: super::VmError) -> bool {
        if self.resource_handoff_pending() {
            self.cleanup.deferred_cancel = Some(error);
            return true;
        }
        if self.cleanup.exits.is_empty() {
            return false;
        }
        if self.cleanup.pending_failure.is_none() {
            self.cleanup.pending_failure = Some(error);
        }
        true
    }

    pub(super) fn register_cleanup(
        &mut self,
        resource: u16,
        disposer: u16,
        family: u16,
    ) -> Result<(), super::VmError> {
        let value = self.read_operand(resource)?.clone();
        let family = CleanupFamily::from_u16(family).ok_or_else(|| {
            super::diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Invalid cleanup family",
            )
        })?;
        if family == CleanupFamily::Process && !self.process_resource_owned(&value) {
            return Err(self.runtime_error(
                fpas_diagnostics::codes::RUNTIME_RESOURCE_RELEASE_FAILURE,
                "Cannot register an unknown process resource",
                "Register a process handle that was issued by this VM.",
            ));
        }
        let Value::Function(disposer) = self.read_operand(disposer)? else {
            return Err(super::diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Cleanup disposer is not callable",
            ));
        };
        let disposer = disposer.clone();
        self.require_function_task_owner(&disposer)?;
        let Some(scope) = self.cleanup.scopes.last_mut() else {
            return Err(super::diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Cleanup registration has no open scope",
            ));
        };
        scope.resources.push(CleanupResource {
            value,
            disposer,
            family,
        });
        self.cleanup.initializers.pop();
        if let Some(error) = self.cleanup.deferred_cancel.take() {
            self.begin_cleanup_failure(error)?;
        }
        Ok(())
    }

    pub(super) fn begin_cleanup_exit(&mut self, protected: bool) -> Result<(), super::VmError> {
        let Some(scope) = self.cleanup.scopes.pop() else {
            return Err(super::diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Cleanup exit has no open scope",
            ));
        };
        self.cleanup.initializers.truncate(scope.initializer_count);
        self.callback_continuations.truncate(scope.callback_depth);
        self.unwind_cleanup_frames(scope.frame_depth);
        let protected = protected || self.cleanup.exits.iter().any(|exit| exit.protected);
        self.cleanup.exits.push(CleanupExit {
            frame_depth: scope.frame_depth,
            resources: scope.resources,
            awaiting_depth: None,
            recovery_depth: None,
            scope_base: self.cleanup.scopes.len(),
            callback_base: self.callback_continuations.len(),
            family: CleanupFamily::Custom,
            result: None,
            current_resource: None,
            protected,
            failed: false,
        });
        Ok(())
    }

    fn unwind_cleanup_frames(&mut self, depth: usize) {
        while self.call_stack.len() > depth {
            let Some(frame) = self.call_stack.pop() else {
                break;
            };
            self.release_registers(self.base);
            self.restore_caller_window(&frame);
            self.function = frame.function;
            self.ip = frame.ip;
            self.base = frame.base;
        }
    }

    pub(super) fn cleanup_resume_pending(&self) -> bool {
        self.cleanup
            .exits
            .last()
            .is_some_and(|exit| exit.awaiting_depth.is_none() || exit.recovery_depth.is_some())
            || (self.cleanup.exits.is_empty() && self.cleanup.pending_failure.is_some())
    }

    pub(super) fn advance_cleanup(&mut self) -> Result<(), super::VmError> {
        if self.cleanup.exits.is_empty() {
            if let Some(scope) = self.cleanup.scopes.last() {
                if self.task_scopes.len() > scope.task_scope_depth {
                    return self.exit_task_scope(true);
                }
                return self.begin_cleanup_exit(false);
            }
            return self.cleanup.pending_failure.take().map_or(Ok(()), Err);
        }
        let Some(exit) = self.cleanup.exits.last_mut() else {
            unreachable!()
        };
        if let Some(depth) = exit.recovery_depth {
            if self.cleanup.scopes.len() > exit.scope_base {
                if self
                    .cleanup
                    .scopes
                    .last()
                    .is_some_and(|scope| self.task_scopes.len() > scope.task_scope_depth)
                {
                    return self.exit_task_scope(true);
                }
                return self.begin_cleanup_exit(false);
            }
            exit.recovery_depth = None;
            exit.awaiting_depth = None;
            let callback_base = exit.callback_base;
            self.callback_continuations.truncate(callback_base);
            self.unwind_cleanup_frames(depth);
            return Ok(());
        }
        let returned = exit
            .result
            .take()
            .map(|result| (exit.family, exit.current_resource.take(), result));
        if let Some((family, resource, Value::ResultError(error))) = returned
            && family != CleanupFamily::Custom
            && !already_closed(family, &error)
            && !(family == CleanupFamily::Process
                && process_invalid_argument(&error)
                && resource
                    .as_ref()
                    .is_some_and(|value| self.process_resource_closed(value)))
        {
            let error = self.runtime_error(
                fpas_diagnostics::codes::RUNTIME_RESOURCE_RELEASE_FAILURE,
                format!("Resource release failed: {error}"),
                "Inspect the resource release failure and its original operation.",
            );
            self.begin_cleanup_failure(error)?;
            return Ok(());
        }
        let Some(exit) = self.cleanup.exits.last_mut() else {
            return Ok(());
        };
        let Some(resource) = exit.resources.pop() else {
            let failed = exit.failed;
            self.cleanup.exits.pop();
            if failed && let Some(parent) = self.cleanup.exits.last_mut() {
                parent.failed = true;
                parent.recovery_depth = parent.awaiting_depth.map(|depth| depth.saturating_sub(1));
            }
            return Ok(());
        };
        exit.result = None;
        exit.family = resource.family;
        exit.current_resource = Some(resource.value.clone());
        self.enter_cleanup_callable(&resource.disposer, resource.value)?;
        let depth = self.call_stack.len();
        if let Some(exit) = self.cleanup.exits.last_mut() {
            exit.awaiting_depth = Some(depth);
        }
        Ok(())
    }

    pub(super) fn cleanup_accepts_return(&self) -> bool {
        self.cleanup
            .exits
            .last()
            .is_some_and(|exit| exit.awaiting_depth == Some(self.call_stack.len()))
    }

    pub(super) fn accept_cleanup_return(&mut self, value: Value) {
        if let Some(exit) = self.cleanup.exits.last_mut() {
            exit.awaiting_depth = None;
            exit.result = Some(value);
        }
    }

    pub(super) fn begin_cleanup_failure(
        &mut self,
        error: super::VmError,
    ) -> Result<(), super::VmError> {
        let error = if self.has_cleanup_obligations() {
            let error = self.capture_failure(error);
            self.capture_root_propagation(&error);
            error
        } else {
            error
        };
        if let Some(exit) = self.cleanup.exits.last_mut() {
            exit.failed = true;
            exit.recovery_depth = exit.awaiting_depth.map(|depth| depth.saturating_sub(1));
            let protected = exit.protected;
            self.record_cleanup_failure(error, protected);
            return Ok(());
        }
        if self.cleanup.pending_failure.is_some() {
            self.record_cleanup_failure(error, false);
            return Ok(());
        }
        if self.cleanup.scopes.is_empty() {
            return Err(error);
        }
        self.cleanup.pending_failure = Some(error);
        Ok(())
    }

    fn record_cleanup_failure(&mut self, error: super::VmError, protected: bool) {
        use fpas_diagnostics::codes::{RUNTIME_TASK_CANCELLED, RUNTIME_VM_SHUTDOWN};

        let is_cancellation = |error: &super::VmError| {
            matches!(error.code, RUNTIME_TASK_CANCELLED | RUNTIME_VM_SHUTDOWN)
        };
        match self.cleanup.pending_failure.as_ref() {
            Some(primary) if is_cancellation(primary) && !is_cancellation(&error) => {
                self.cleanup.pending_failure = Some(error);
            }
            Some(_) if !is_cancellation(&error) => self.report_cleanup_diagnostic(&error),
            Some(_) => {}
            None if protected => self.report_cleanup_diagnostic(&error),
            None => self.cleanup.pending_failure = Some(error),
        }
    }

    fn report_cleanup_diagnostic(&self, error: &super::VmError) {
        self.hosted
            .supplemental_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((**error).clone());
        let receiver = self
            .hosted
            .supplemental_diagnostic
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(receiver) = receiver {
            receiver(error);
        }
    }

    pub(in crate::execution::vm) fn has_cleanup_from_depth(&self, depth: usize) -> bool {
        self.cleanup.initializers.iter().any(|initializer| {
            (initializer.ready && initializer.depth >= depth) || initializer.armed
        }) || self.cleanup.exits.iter().any(|exit| {
            exit.frame_depth >= depth || exit.awaiting_depth.is_some_and(|d| d >= depth)
        }) || self
            .cleanup
            .scopes
            .iter()
            .any(|scope| scope.frame_depth >= depth)
    }
}

fn already_closed(family: CleanupFamily, error: &Value) -> bool {
    let Value::Enum(value) = error else {
        return false;
    };
    let layout = &value.body().layout;
    match family {
        CleanupFamily::File => {
            layout.type_name == "Std.Io.Fs.FsError" && layout.variant == "Closed"
        }
        CleanupFamily::Socket | CleanupFamily::Listener => {
            layout.type_name == "Std.Net.Sockets.NetError" && layout.variant == "Closed"
        }
        CleanupFamily::Acp => {
            layout.type_name == "Std.Ai.Acp.Types.AcpError" && layout.variant == "Closed"
        }
        CleanupFamily::Mcp | CleanupFamily::McpEndpoint => {
            layout.type_name == "Std.Ai.Mcp.Types.McpError" && layout.variant == "Closed"
        }
        CleanupFamily::HttpStream
            if layout.type_name == "Std.Net.Http.Types.HttpError"
                && layout.variant == "Transport" =>
        {
            value
                .body()
                .values
                .first()
                .is_some_and(|cause| already_closed(CleanupFamily::Socket, cause))
        }
        _ => false,
    }
}

fn process_invalid_argument(error: &Value) -> bool {
    matches!(error, Value::Enum(value) if value.body().layout.type_name == "Std.Os.Proc.ProcError"
        && value.body().layout.variant == "InvalidArgument")
}

fn producer_succeeded(mut value: &Value) -> bool {
    loop {
        match value {
            Value::ResultOk(inner) | Value::OptionSome(inner) => value = inner,
            Value::ResultError(_) | Value::OptionNone => return false,
            _ => return true,
        }
    }
}
