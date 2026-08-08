//! Register task pool execution and inline progress for synchronous waits.

use std::sync::Arc;

use super::{TaskScheduler, TaskState};
use crate::vm::VmError;
use crate::vm::worker::Worker;

pub(in crate::vm) fn pool_loop(
    template: &Worker,
    scheduler: Arc<TaskScheduler>,
) -> Result<(), VmError> {
    while let Some(task) = scheduler.dequeue() {
        let mut worker = template.worker_for_task(task);
        match worker.run_task() {
            Ok(Some(value)) => {
                if worker.retain_result {
                    scheduler.store_result(worker.task_id, value);
                }
            }
            Ok(None) => {}
            Err(error) => {
                if worker.retain_result {
                    scheduler.store_failure(worker.task_id, error.clone());
                }
                scheduler.fail(error.clone());
                return Err(error);
            }
        }
    }
    Ok(())
}

pub(super) fn run_helped(
    parent: &Worker,
    task: TaskState,
    scheduler: Arc<TaskScheduler>,
) -> Result<(), VmError> {
    let mut worker = parent.worker_for_task(task);
    match worker.run_task()? {
        Some(value) if worker.retain_result => scheduler.store_result(worker.task_id, value),
        _ => {}
    }
    Ok(())
}
