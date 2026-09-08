//! Release a joined group's retained child results in the scheduler's lock order.

use super::TaskScheduler;
use crate::vm::tasks::groups::{ClosedGroup, GroupFailure};
use crate::vm::{TaskResultState, VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;
use std::collections::HashMap;

impl TaskScheduler {
    /// Collect terminal reports and discard child results only after the complete group joins.
    pub(in crate::vm) fn poll_group_close(
        &self,
        id: u64,
    ) -> Result<Option<Vec<GroupFailure>>, VmError> {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        // Shutdown can publish synthetic failures before running workers have exited.
        // Check under the results lock so those reports cannot masquerade as a joined group.
        if self.is_shutdown() {
            return Err(self
                .first_error()
                .unwrap_or_else(|| self.shutdown_error(id)));
        }
        let Some(group) = self.groups.take_closed(id).map_err(|message| {
            runtime_error(
                RUNTIME_INVALID_TASK,
                message,
                "Close the TaskGroup from its creating task before the runtime shuts down.",
                SourceLocation::new(1, 1),
            )
        })?
        else {
            return Ok(None);
        };
        Ok(Some(self.release_group_results(group, &mut results)))
    }

    /// Release a group without cancellation when all registered children are already terminal.
    pub(in crate::vm) fn try_close_completed_group(
        &self,
        id: u64,
        owner: u64,
    ) -> Result<Option<Vec<GroupFailure>>, VmError> {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        if self.is_shutdown() {
            return Err(self
                .first_error()
                .unwrap_or_else(|| self.shutdown_error(id)));
        }
        let Some(group) = self
            .groups
            .try_take_completed(id, owner)
            .map_err(|message| {
                runtime_error(
                    RUNTIME_INVALID_TASK,
                    message,
                    "Try from the creating task after every child has completed.",
                    SourceLocation::new(1, 1),
                )
            })?
        else {
            return Ok(None);
        };
        Ok(Some(self.release_group_results(group, &mut results)))
    }

    fn release_group_results(
        &self,
        group: ClosedGroup,
        results: &mut HashMap<u64, TaskResultState>,
    ) -> Vec<GroupFailure> {
        let mut completions = self.completions.lock().unwrap_or_else(|e| e.into_inner());
        for task in group.tasks {
            results.remove(&task);
            completions.insert(task);
        }
        group.failures
    }
}
