//! Task scopes: frame-owned children, normal and cancelling exits, and cancellation checks.
//!
//! Documentation: `docs/pascal/language/concurrency/task-scopes.md`.

mod failures;
mod registry;
mod unwind;

pub(in crate::execution::vm) use failures::ScopeFailure;
pub(in crate::execution::vm) use registry::{ScopeControl, ScopeRegistry};

use std::sync::Arc;
use std::time::{Duration, Instant};

use fpas_diagnostics::codes::{RUNTIME_TASK_CANCELLED, RUNTIME_VM_SHUTDOWN};

use crate::execution::vm::shared::wakeups::WakeSignal;
use crate::execution::vm::tasks::TaskSuspension;
use crate::execution::vm::{VmError, diagnostics, worker::Worker};

/// A task scope opened by one call frame of the current task.
#[derive(Clone)]
pub(in crate::execution::vm) struct OpenScope {
    id: u64,
    /// Call-stack depth of the frame that opened the scope.
    frame_depth: usize,
    control: Arc<ScopeControl>,
}

impl Worker {
    /// Open a task scope owned by the current call frame.
    pub(in crate::execution::vm) fn enter_task_scope(&mut self) -> Result<(), VmError> {
        let (id, control) = self.scheduler_ref()?.scopes.open();
        self.scope_failure_sequence.set(None);
        self.task_scopes.push(OpenScope {
            id,
            frame_depth: self.call_stack.len(),
            control,
        });
        Ok(())
    }

    /// Scope that owns a `go` of the current frame; calls and callbacks never inherit one.
    pub(in crate::execution::vm) fn spawning_scope(&self) -> Option<u64> {
        self.task_scopes
            .last()
            .filter(|scope| scope.frame_depth == self.call_stack.len())
            .map(|scope| scope.id)
    }

    /// Whether a frame at `depth` or deeper has an open task scope.
    pub(in crate::execution::vm) fn has_task_scope_from_depth(&self, depth: usize) -> bool {
        self.task_scopes
            .iter()
            .any(|scope| scope.frame_depth >= depth)
    }

    /// Leave the innermost scope: optionally cancel its children, then wait for all of them.
    pub(in crate::execution::vm) fn exit_task_scope(
        &mut self,
        cancel: bool,
    ) -> Result<(), VmError> {
        let scheduler = Arc::clone(self.scheduler_ref()?);
        let scope = self.innermost_scope()?;
        if cancel {
            scheduler.scopes.cancel(scope);
        }
        if self.debug_tasks || self.task_id != 0 {
            if !self.poll_scope_exit()? {
                self.task_suspension = Some(TaskSuspension::ScopeExit);
                self.suspend_requested = true;
                if !self.debug_tasks {
                    self.park_pool_suspension()?;
                }
            }
            return Ok(());
        }
        loop {
            let signal = WakeSignal::new();
            // Keeps the wake subscription alive while this probe parks.
            let _registration = scheduler.subscribe(&signal);
            if self.poll_scope_exit()? {
                return Ok(());
            }
            self.check_task_cancellation()?;
            let _wait = self.native_wait(Vec::new())?;
            signal.wait(Duration::from_millis(10));
        }
    }

    /// Close the innermost scope once its children have finished and raise its failure.
    pub(in crate::execution::vm) fn poll_scope_exit(&mut self) -> Result<bool, VmError> {
        let scheduler = Arc::clone(self.scheduler_ref()?);
        self.reject_scope_wait_after_shutdown(&scheduler)?;
        let Some(failures) = scheduler.poll_scope_close(self.innermost_scope()?) else {
            return Ok(false);
        };
        self.task_scopes.pop();
        match failures::scope_outcome(failures, None) {
            None => Ok(true),
            Some((sequence, error)) => {
                self.scope_failure_sequence.set(Some(sequence));
                Err(error)
            }
        }
    }

    /// Inherit cancellation from a synchronous caller without inheriting spawn ownership.
    pub(in crate::execution::vm) fn inherit_scope_cancellation(&mut self, caller: &Self) {
        self.owner_scope = caller.owner_scope.clone();
        self.enclosing_scope_controls
            .clone_from(&caller.enclosing_scope_controls);
        self.enclosing_scope_controls.extend(
            caller
                .task_scopes
                .iter()
                .map(|scope| Arc::clone(&scope.control)),
        );
        self.scope_failure_sequence.set(None);
    }

    /// Stop this task when its owning scope was cancelled or an enclosing scope failed.
    #[inline(always)]
    pub(in crate::execution::vm) fn check_task_cancellation(&self) -> Result<(), VmError> {
        if self.cleanup_masks_cancellation() {
            return Ok(());
        }
        if self
            .scheduler
            .as_ref()
            .is_some_and(|scheduler| scheduler.is_aborted())
        {
            return Err(self.runtime_error(
                RUNTIME_VM_SHUTDOWN,
                "Register VM execution was canceled",
                "Allow active resource cleanup to finish before stopping the runtime.",
            ));
        }
        // Runs before every instruction; code without scope ownership has nothing to observe.
        if self.has_task_scope_ownership() {
            self.check_scope_cancellation()
        } else {
            Ok(())
        }
    }

    fn check_scope_cancellation(&self) -> Result<(), VmError> {
        if self
            .owner_scope
            .as_ref()
            .is_some_and(|control| control.is_cancelled())
        {
            return Err(self.runtime_error(
                RUNTIME_TASK_CANCELLED,
                format!(
                    "Task {} was cancelled because its task scope was left early or failed",
                    self.task_id
                ),
                "A task scope cancels its remaining children when it is left before `end scope` or a child fails. Wait for required results inside the scope.",
            ));
        }
        if self
            .task_scopes
            .iter()
            .map(|scope| &scope.control)
            .chain(&self.enclosing_scope_controls)
            .any(|control| control.has_failed())
        {
            return Err(self.runtime_error(
                RUNTIME_TASK_CANCELLED,
                "Task scope body was cancelled because a child task failed",
                "The scope reports the child's failure when it is left.",
            ));
        }
        Ok(())
    }

    /// Whether this task must observe scope cancellation while it waits.
    #[inline(always)]
    pub(in crate::execution::vm) fn has_task_scope_ownership(&self) -> bool {
        self.owner_scope.is_some()
            || !self.task_scopes.is_empty()
            || !self.enclosing_scope_controls.is_empty()
    }

    /// Preserve the identity of a child failure raised by a task wait.
    pub(in crate::execution::vm) fn observe_task_failure(
        &self,
        id: u64,
        error: VmError,
    ) -> VmError {
        self.observed_task_failure.set(Some(id));
        self.scope_failure_sequence.set(
            self.scheduler
                .as_ref()
                .and_then(|scheduler| scheduler.scopes.failure_sequence(id)),
        );
        error
    }

    /// Wait for scheduler changes while keeping scope cancellation observable.
    pub(in crate::execution::vm) fn wait_observing_scopes(&self) -> Result<(), VmError> {
        let signal = WakeSignal::new();
        let _registration = self.scheduler_ref()?.subscribe(&signal);
        self.check_task_cancellation()?;
        let _wait = self.native_wait(Vec::new())?;
        signal.wait(Duration::from_millis(10));
        Ok(())
    }

    /// Sleep on a blocking thread in short slices so a failed scope stops its body promptly.
    pub(in crate::execution::vm) fn sleep_observing_scopes(
        &self,
        milliseconds: u64,
    ) -> Result<(), VmError> {
        let deadline = Instant::now() + Duration::from_millis(milliseconds);
        loop {
            self.check_task_cancellation()?;
            let now = Instant::now();
            if now >= deadline {
                return Ok(());
            }
            std::thread::sleep((deadline - now).min(Duration::from_millis(10)));
        }
    }

    fn innermost_scope(&self) -> Result<u64, VmError> {
        self.task_scopes
            .last()
            .map(|scope| scope.id)
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Task scope exit has no open task scope",
                )
            })
    }

    fn reject_scope_wait_after_shutdown(
        &self,
        scheduler: &crate::execution::vm::tasks::TaskScheduler,
    ) -> Result<(), VmError> {
        if !scheduler.is_shutdown() {
            return Ok(());
        }
        Err(scheduler.first_error().unwrap_or_else(|| {
            self.runtime_error(
                RUNTIME_VM_SHUTDOWN,
                "Task scope was interrupted by runtime shutdown",
                "Keep the program running until every task scope has ended.",
            )
        }))
    }
}
