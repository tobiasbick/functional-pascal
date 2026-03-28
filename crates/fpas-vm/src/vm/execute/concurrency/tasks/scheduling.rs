use super::super::super::super::{TIMESLICE, TaskContext, Vm};
use fpas_bytecode::Value;

impl Vm {
    pub(in super::super::super) fn complete_current_task(&mut self, value: Value) {
        self.task_results.insert(self.current_task_id, value);
    }

    pub(in super::super::super) fn schedule_next_task(&mut self) -> bool {
        if self.tasks.is_empty() {
            return false;
        }

        let task = self.tasks.remove(0);
        self.load_task_state(task);
        self.instructions_until_yield = TIMESLICE;
        true
    }

    pub(in super::super::super) fn maybe_timeslice_yield(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        self.instructions_until_yield = self.instructions_until_yield.saturating_sub(1);
        if self.instructions_until_yield == 0 {
            self.yield_current_task();
        }
    }

    pub(super) fn yield_current_task(&mut self) {
        let saved = self.save_current_task_state();
        self.tasks.push(saved);
        let next = self.tasks.remove(0);
        self.load_task_state(next);
        self.instructions_until_yield = TIMESLICE;
    }

    fn save_current_task_state(&mut self) -> TaskContext {
        TaskContext {
            id: self.current_task_id,
            ip: self.ip,
            stack: std::mem::take(&mut self.stack),
            call_stack: std::mem::take(&mut self.call_stack),
        }
    }

    fn load_task_state(&mut self, task: TaskContext) {
        self.current_task_id = task.id;
        self.ip = task.ip;
        self.stack = task.stack;
        self.call_stack = task.call_stack;
    }
}
