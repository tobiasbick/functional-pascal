//! Instruction-budget preemption for spawned tasks.
//!
//! The main task (id `0`) never uses the shared ready queue; only pool workers run spawned tasks
//! that may be saved and re-queued here. See `docs/pascal/language/concurrency/README.md`
//! `docs/pascal/language/concurrency/README.md`.

use crate::vm::{TIMESLICE, Worker};

impl Worker {
    /// Yield to allow other tasks to run (timeslice preemption).
    ///
    /// Decrements the instruction counter and, when the timeslice is
    /// exhausted, saves the current task and picks up the next one from
    /// the shared queue. The main task (`id == 0`) and sync calls never yield.
    pub(in crate::vm::execute) fn maybe_timeslice_yield(&mut self) {
        // Main-task bytecode never enters the shared ready queue; skip the
        // counter entirely on that hot path.
        if self.sync_call_depth > 0 || self.current_task_id == 0 {
            return;
        }
        self.instructions_until_yield = self.instructions_until_yield.saturating_sub(1);
        if self.instructions_until_yield == 0 {
            self.instructions_until_yield = TIMESLICE;
            // Try to pick up another task; if none, continue current.
            self.switch_to_next_ready_task();
        }
    }
}
