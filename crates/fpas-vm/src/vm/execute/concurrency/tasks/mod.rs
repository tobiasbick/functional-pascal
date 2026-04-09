use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;

mod scheduling;
mod spawn;
mod wait;

impl Worker {
    pub(super) fn exec_yield(&mut self) -> bool {
        if self.sync_call_depth > 0 {
            return false;
        }
        if self.current_task_id == 0 {
            // The main task must never be enqueued — pool workers must not steal it.
            // Yield CPU time so pool workers can make progress on spawned tasks.
            std::thread::yield_now();
            return false;
        }
        self.switch_to_next_ready_task()
    }

    fn pop_task_id(&mut self, line: SourceLocation) -> Result<u64, VmError> {
        wait::pop_task_id(self, line)
    }

    pub(in crate::vm::execute) fn switch_to_next_ready_task(&mut self) -> bool {
        if let Some(next) = self.shared.try_dequeue_task() {
            let saved = self.save_task();
            self.shared.enqueue_task(saved);
            self.load_task(next);
            true
        } else {
            false
        }
    }
}
