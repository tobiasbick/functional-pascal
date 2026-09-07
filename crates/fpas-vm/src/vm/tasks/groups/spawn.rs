//! Start a retained child after registering its lifecycle owner.

use super::*;
use crate::vm::tasks::TaskState;
use crate::vm::tasks::supervision::{RetryPolicy, SupervisedTask};

impl Worker {
    /// Register ownership before preparing captures and publishing the child's task handle.
    pub(super) fn start_group_task(
        &self,
        group: u64,
        work: &Value,
        policy: Option<RetryPolicy>,
    ) -> Result<Value, VmError> {
        let function = self.validate_callback(work, 1)?;
        if function.task_bound {
            return Err(self.group_error("Cannot start a task-bound worker in another task"));
        }
        let scheduler = self.scheduler_ref()?;
        let id = scheduler.alloc_id();
        let token = scheduler
            .groups
            .enroll(group, self.task_id, id)
            .map_err(|e| self.group_error(e))?;
        let info = &self.executable.executable().functions[usize::from(function.function.get())];
        scheduler.register_result(id);
        let mut task = TaskState::entry(id, function, info, [Value::OpaqueHandle(token)], true);
        task.supervision =
            policy.map(|policy| Box::new(SupervisedTask::new(function.clone(), token, policy)));
        scheduler.enqueue(task);
        Ok(Value::Task(id))
    }
}
