//! Stopped-state task observation without scheduler admission or dispatch.

use super::super::super::types::DebugTask;
use super::DebugTaskRuntime;

impl DebugTaskRuntime {
    /// Capture the current task catalog without draining queued spawns or waking waiters.
    pub(in crate::vm::debug) fn catalog(&self) -> Vec<DebugTask> {
        self.tasks
            .iter()
            .map(|(&id, slot)| DebugTask {
                id,
                name: slot.name.clone(),
                state: slot.state,
                inspectable: slot.state.is_inspectable(),
                paused: slot.paused,
            })
            .collect()
    }

    /// Enqueue one function as a scheduler-queued spawn that resume must admit.
    #[cfg(test)]
    pub(in crate::vm::debug) fn test_enqueue_pending_task(
        &self,
        function: fpas_bytecode::FunctionId,
    ) {
        let Some(template) = self.tasks.get(&0) else {
            return;
        };
        let Some(info) = template
            .worker
            .executable
            .executable()
            .functions
            .get(usize::from(function.get()))
        else {
            return;
        };
        let register_count = usize::from(info.register_count);
        let ip = usize::try_from(info.code.start.get()).unwrap_or(0);
        self.scheduler.enqueue(crate::vm::tasks::TaskState {
            id: self.scheduler.alloc_id(),
            function,
            ip,
            base: 0,
            registers: vec![fpas_bytecode::Value::Unit; register_count],
            register_initialized: vec![false; register_count],
            frames: Vec::new(),
            retain_result: false,
            instruction_count: 0,
            suppressed_initializers: Vec::new(),
        });
    }
}
