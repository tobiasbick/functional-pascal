//! Save cooperative waits without retaining an inline helper's caller stack.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use crate::vm::{VmError, worker::Worker};

impl Worker {
    /// Resume a saved wait before executing the instruction following its intrinsic.
    pub(in crate::vm) fn resume_pool_suspension(&mut self) -> Result<bool, VmError> {
        if self.poll_task_suspension()? {
            return Ok(true);
        }
        self.park_pool_suspension()?;
        Ok(false)
    }

    /// Transfer pending operation ownership to the shared timer driver.
    pub(in crate::vm) fn park_pool_suspension(&mut self) -> Result<(), VmError> {
        let scheduler = std::sync::Arc::clone(self.scheduler_ref()?);
        let state = self.take_task_state();
        scheduler.schedule(state, 1);
        self.suspend_requested = true;
        Ok(())
    }
}
