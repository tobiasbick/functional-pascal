//! Validate final task results and release unused cases before terminal publication.

use std::sync::{Arc, Weak};

use crate::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::TaskScheduler;
use crate::execution::vm::tasks::selection::CaseRegistry;
use crate::execution::vm::{TaskResultState, VmError, runtime_error};

impl TaskScheduler {
    /// Link cleanup without retaining the session or its values from the scheduler.
    pub(in crate::execution::vm) fn attach_cases(&self, cases: &Arc<CaseRegistry>) {
        let _ = self.cases.set(Arc::downgrade(cases));
    }

    pub(super) fn close_owner_cases(&self, owner: u64) {
        if let Some(cases) = self.cases.get().and_then(Weak::upgrade) {
            cases.close_owner(owner);
        }
    }

    pub(super) fn close_all_cases(&self) {
        if let Some(cases) = self.cases.get().and_then(Weak::upgrade) {
            cases.close_all();
        }
    }

    pub(super) fn validate_result(value: &Value) -> Result<(), VmError> {
        crate::execution::vm::domain::check_transfer(std::slice::from_ref(value)).map_err(|message| {
            runtime_error(
                RUNTIME_INVALID_TASK,
                message,
                "Task results must contain transferable values without local captures or references.",
                SourceLocation::new(1, 1),
            )
        })
    }

    /// An invalid late completion must not overwrite an already terminal result.
    pub(super) fn reject_result(&self, id: u64, error: VmError) {
        self.close_owner_cases(id);
        let mut results = self
            .results
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !matches!(results.get(&id), Some(TaskResultState::Pending)) {
            return;
        }
        let owned = self.groups.complete(
            id,
            Some(crate::execution::vm::tasks::groups::GroupFailure::runtime(
                id, &error,
            )),
        ) | self.scopes.complete(id, Some(&error));
        results.insert(id, TaskResultState::Failed(Box::new(error.clone())));
        if let Some(capture) = self.failure_capture.get() {
            capture.task_finished(id, Some(&error));
        }
        drop(results);
        self.notify_result_change();
        if !owned && !self.is_shutdown() {
            self.fail(error);
        }
    }
}

#[cfg(test)]
mod tests;
