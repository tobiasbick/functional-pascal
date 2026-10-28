//! Leave every open task scope after a panic, runtime error, or cancellation ended the body.
//!
//! Documentation: `docs/pascal/language/concurrency/task-scopes.md`.

use std::sync::Arc;
use std::time::Duration;

use crate::execution::vm::shared::wakeups::WakeSignal;
use crate::execution::vm::tasks::TaskSuspension;
use crate::execution::vm::{VmError, worker::Worker};

use super::failures::{ScopeFailure, scope_outcome};

impl Worker {
    /// Cancel every open scope and suspend until their children finish.
    ///
    /// Returns `error` unchanged when no scope is open or the runtime is already aborting.
    pub(in crate::execution::vm) fn begin_scope_unwind(
        &mut self,
        error: VmError,
    ) -> Result<(), VmError> {
        let Some(scheduler) = self.scheduler.clone() else {
            return Err(error);
        };
        if self.task_scopes.is_empty() || scheduler.is_aborted() {
            self.task_scopes.clear();
            return Err(error);
        }
        let sequence = self
            .scope_failure_sequence
            .take()
            .unwrap_or_else(|| scheduler.scopes.next_sequence());
        for scope in &self.task_scopes {
            scheduler.scopes.cancel(scope.id);
        }
        self.task_suspension = Some(TaskSuspension::ScopeUnwind {
            error,
            sequence,
            failures: Vec::new(),
        });
        self.suspend_requested = true;
        Ok(())
    }

    /// Close scopes innermost first; once all are closed, fail with the scope outcome.
    pub(in crate::execution::vm) fn poll_scope_unwind(
        &mut self,
        error: VmError,
        sequence: u64,
        mut failures: Vec<ScopeFailure>,
    ) -> Result<bool, VmError> {
        let scheduler = Arc::clone(self.scheduler_ref()?);
        while let Some(scope) = self.task_scopes.last() {
            if scheduler.is_aborted() {
                self.task_scopes.clear();
                return Err(error);
            }
            let Some(closed) = scheduler.poll_scope_close(scope.id) else {
                self.task_suspension = Some(TaskSuspension::ScopeUnwind {
                    error,
                    sequence,
                    failures,
                });
                return Ok(false);
            };
            failures.extend(closed);
            self.task_scopes.pop();
        }
        let (sequence, error) =
            scope_outcome(failures, Some((sequence, error.clone()))).unwrap_or((sequence, error));
        self.scope_failure_sequence.set(Some(sequence));
        Err(error)
    }

    /// Leave every open scope on the current thread and return the task's final error.
    ///
    /// Used by the main task and synchronous callbacks, which never park on the pool.
    pub(in crate::execution::vm) fn unwind_task_scopes_blocking(
        &mut self,
        error: VmError,
    ) -> VmError {
        if let Err(error) = self.begin_scope_unwind(error) {
            return error;
        }
        let Some(scheduler) = self.scheduler.clone() else {
            unreachable!("scope unwinding requires a scheduler")
        };
        loop {
            let signal = WakeSignal::new();
            // Keeps the wake subscription alive while this probe parks.
            let _registration = scheduler.subscribe(&signal);
            let Some(TaskSuspension::ScopeUnwind {
                error,
                sequence,
                failures,
            }) = self.task_suspension.take()
            else {
                unreachable!("scope unwinding keeps its suspension until it completes")
            };
            match self.poll_scope_unwind(error, sequence, failures) {
                Ok(false) => {
                    signal.wait(Duration::from_millis(10));
                }
                Ok(true) => unreachable!("scope unwinding always ends with the task's error"),
                Err(error) => {
                    self.suspend_requested = false;
                    return error;
                }
            }
        }
    }
}
