//! Register task pool execution.

use std::sync::Arc;

mod suspension;

use super::TaskScheduler;
#[cfg(test)]
use super::TaskState;
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

/// Run one dequeued task on the caller's thread; tests use it to drive tasks deterministically.
#[cfg(test)]
pub(super) fn run_helped(
    parent: &Worker,
    task: TaskState,
    scheduler: Arc<TaskScheduler>,
) -> Result<(), VmError> {
    run_to_completion(parent.worker_for_task(task), &scheduler)
}

fn run_to_completion(mut worker: Worker, scheduler: &TaskScheduler) -> Result<(), VmError> {
    let access = worker.execution_access()?;
    let _capture_panic = worker.capture_pool_guard();
    let attempt = (|| {
        if !worker.supervised_ready()? || !worker.resume_pool_suspension()? {
            return Ok(None);
        }
        worker.run_task()
    })();
    // A body that ends with open task scopes first waits for their cancelled children.
    let attempt = match attempt {
        Err(error) => {
            let error = worker.capture_failure(error);
            worker.capture_root_propagation(&error);
            match worker.begin_scope_unwind(error) {
                Ok(()) => worker.park_pool_suspension().map(|()| None),
                Err(error) => Err(error),
            }
        }
        other => other,
    };
    let _outcome_roots = worker.domain.root_values(
        fpas_bytecode::RootKind::Native,
        match &attempt {
            Ok(Some(value)) => vec![value.clone()],
            _ => Vec::new(),
        },
    );
    let outcome = match attempt {
        Ok(Some(value)) => worker
            .finish_owned_groups(Ok(value))
            .and_then(|value| value.map_or(Ok(None), |value| worker.supervised_outcome(Ok(value)))),
        Err(error) => match worker.finish_owned_groups(Err(error)) {
            Err(error) => worker.supervised_outcome(Err(error)),
            outcome => outcome,
        },
        Ok(None) => Ok(None),
    };
    let result = match outcome {
        Ok(Some(value)) => {
            if worker.retain_result {
                scheduler.store_result(worker.task_id, value);
            } else if let Some(capture) = scheduler.failure_capture.get() {
                capture.release_task(worker.task_id);
            }
            Ok(())
        }
        Ok(None) => {
            if matches!(
                worker.task_suspension,
                Some(super::TaskSuspension::OwnerCleanup { .. })
            ) {
                worker.park_pool_suspension()?;
            }
            if scheduler.is_aborted() {
                worker.capture_aborted_task();
            }
            Ok(())
        }
        Err(error) => {
            let error = worker.capture_failure(error);
            if worker.retain_result {
                let grouped = scheduler.store_failure(worker.task_id, error.clone());
                if grouped && !scheduler.is_aborted() {
                    worker.retire_execution();
                    return Ok(());
                }
            }
            scheduler.fail(error.clone());
            Err(error)
        }
    };
    if worker.suspend_requested && worker.execution.is_some() {
        if worker.publish_execution(&[]).is_ok() {
            access.safe_point();
        }
    } else if !worker.suspend_requested {
        worker.retire_execution();
    }
    result
}
