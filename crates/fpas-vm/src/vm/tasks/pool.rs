//! Register task pool execution and inline progress for synchronous waits.

use std::sync::Arc;

mod suspension;

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
    let attempt = (|| {
        if !worker.supervised_ready()? || !worker.resume_pool_suspension()? {
            return Ok(None);
        }
        worker.run_task()
    })();
    let outcome = match attempt {
        Ok(Some(value)) => worker.supervised_outcome(Ok(value)),
        Err(error) => worker.supervised_outcome(Err(error)),
        Ok(None) => Ok(None),
    };
    match outcome {
        Ok(Some(value)) => {
            if worker.retain_result {
                scheduler.store_result(worker.task_id, value);
            }
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => {
            if worker.retain_result {
                let grouped = scheduler.store_failure(worker.task_id, error.clone());
                if grouped && !scheduler.is_aborted() {
                    return Ok(());
                }
            }
            scheduler.fail(error.clone());
            Err(error)
        }
    }
}
