//! Register task pool execution and inline progress for synchronous waits.

use std::sync::Arc;

use super::{RegisterTaskState, TaskScheduler};
use crate::vm::VmError;
use crate::vm::register::worker::RegisterWorker;

pub(in crate::vm::register) fn pool_loop(
    template: &RegisterWorker,
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
    parent: &RegisterWorker,
    task: RegisterTaskState,
    scheduler: Arc<TaskScheduler>,
) -> Result<(), VmError> {
    let mut worker = parent.worker_for_task(task);
    match worker.run_task()? {
        Some(value) if worker.retain_result => scheduler.store_result(worker.task_id, value),
        _ => {}
    }
    Ok(())
}
