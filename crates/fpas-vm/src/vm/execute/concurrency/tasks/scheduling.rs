use super::super::super::super::{TIMESLICE, Worker};

impl Worker {
    /// Yield to allow other tasks to run (timeslice preemption).
    ///
    /// Decrements the instruction counter and, when the timeslice is
    /// exhausted, saves the current task and picks up the next one from
    /// the shared queue.
    pub(in super::super::super) fn maybe_timeslice_yield(&mut self) {
        if self.sync_call_depth > 0 {
            return;
        }
        self.instructions_until_yield = self.instructions_until_yield.saturating_sub(1);
        if self.instructions_until_yield == 0 {
            self.instructions_until_yield = TIMESLICE;
            // The main task (id 0) must never be enqueued — only pool workers
            // should swap tasks via the shared queue.
            if self.current_task_id == 0 {
                return;
            }
            // Try to pick up another task; if none, continue current.
            if let Some(next) = self.shared.try_dequeue_task() {
                let saved = self.save_task();
                self.shared.enqueue_task(saved);
                self.load_task(next);
            }
        }
    }
}
