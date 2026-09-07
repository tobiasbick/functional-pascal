//! Release a joined group's retained child results in the scheduler's lock order.

use super::TaskScheduler;
use crate::vm::tasks::groups::GroupFailure;
use crate::vm::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

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
        let mut completions = self.completions.lock().unwrap_or_else(|e| e.into_inner());
        for task in group.tasks {
            results.remove(&task);
            completions.insert(task);
        }
        Ok(Some(group.failures))
    }
}
