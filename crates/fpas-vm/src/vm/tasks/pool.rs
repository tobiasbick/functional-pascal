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
        run_to_completion(template.worker_for_task(task), &scheduler)?;
    }
    Ok(())
}

pub(super) fn run_helped(
    parent: &Worker,
    task: TaskState,
    scheduler: Arc<TaskScheduler>,
) -> Result<(), VmError> {
    run_to_completion(parent.worker_for_task(task), &scheduler)
}

fn run_to_completion(mut worker: Worker, scheduler: &TaskScheduler) -> Result<(), VmError> {
    match worker.run_task() {
        Ok(Some(value)) => {
            if worker.retain_result {
                scheduler.store_result(worker.task_id, value);
            }
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => {
            if worker.retain_result {
                scheduler.store_failure(worker.task_id, error.clone());
            }
            scheduler.fail(error.clone());
            Err(error)
        }
    }
}
