//! Save cooperative waits without retaining an inline helper's caller stack.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use crate::execution::vm::{VmError, worker::Worker};

impl Worker {
    /// Resume a saved wait before executing the instruction following its intrinsic.
    pub(in crate::execution::vm) fn resume_pool_suspension(&mut self) -> Result<bool, VmError> {
        if self.poll_task_suspension()? {
            return Ok(true);
        }
        self.park_pool_suspension()?;
        Ok(false)
    }

    /// Yield productive pipe work immediately, or park idle waits on the shared timer.
    pub(in crate::execution::vm) fn park_pool_suspension(&mut self) -> Result<(), VmError> {
        let scheduler = std::sync::Arc::clone(self.scheduler_ref()?);
        crate::profile::add(crate::profile::Counter::TaskParks, 1);
        let productive = matches!(self.task_suspension.as_ref(), Some(super::super::TaskSuspension::Proc(wait)) if wait.made_progress());
        let state = self.take_task_state();
        if productive {
            scheduler.enqueue(state);
        } else {
            scheduler.schedule(state, 1);
        }
        self.suspend_requested = true;
        Ok(())
    }
}
