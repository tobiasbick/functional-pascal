use super::super::super::Worker;
use super::super::super::diagnostics::VmError;
use fpas_bytecode::SourceLocation;

mod scheduling;
mod spawn;
mod wait;

impl Worker {
    pub(super) fn exec_yield(&mut self) {
        if self.sync_call_depth > 0 {
            return;
        }
        if self.current_task_id == 0 {
            // The main task must never be enqueued — pool workers must not steal it.
            // Yield CPU time so pool workers can make progress on spawned tasks.
            std::thread::yield_now();
            return;
        }
        // Non-main tasks: save current and pick up another from the queue.
        if let Some(next) = self.shared.try_dequeue_task() {
            let saved = self.save_task();
            self.shared.enqueue_task(saved);
            self.load_task(next);
        }
    }

    fn pop_task_id(&mut self, line: SourceLocation) -> Result<u64, VmError> {
        wait::pop_task_id(self, line)
    }
}
