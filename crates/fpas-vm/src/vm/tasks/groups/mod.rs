//! Explicit child-task ownership, cooperative cancellation, and failure collection.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

mod close;
mod registry;
mod spawn;
pub(in crate::vm) use registry::{ClosedGroup, GroupFailure, GroupRegistry};

use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{Intrinsic, Register, SourceLocation, TaskIntrinsic, Value};
use std::sync::Arc;

impl Worker {
    /// Route task-group operations through the shared normal/debugger owner state.
    pub(in crate::vm::tasks) fn group_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        args: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Task(operation) = intrinsic else {
            return Ok(None);
        };
        let expected = match operation {
            TaskIntrinsic::CreateTaskGroup => 0,
            TaskIntrinsic::StartTaskInGroup | TaskIntrinsic::CloseTaskGroupWithTimeout => 2,
            TaskIntrinsic::StartSupervisedTask => 4,
            TaskIntrinsic::GetTaskGroupToken
            | TaskIntrinsic::CancelTaskGroup
            | TaskIntrinsic::CloseTaskGroup
            | TaskIntrinsic::TryCloseCompletedTaskGroup => 1,
            _ => return Ok(None),
        };
        if args.len() != expected {
            return Err(
                self.group_error(format!("Task group operation expects {expected} arguments"))
            );
        }
        let scheduler = Arc::clone(self.scheduler_ref()?);
        if operation == TaskIntrinsic::CreateTaskGroup {
            let id = scheduler
                .groups
                .create(self.task_id, || self.hosted.cancellations.create_owned())
                .map_err(|e| self.group_error(e))?;
            return Ok(Some(Some(Value::OpaqueHandle(id))));
        }
        let Value::OpaqueHandle(id) = args[0] else {
            return Err(self.task_type_error("TaskGroup", &args[0]));
        };
        let value = match operation {
            TaskIntrinsic::GetTaskGroupToken => Value::OpaqueHandle(
                scheduler
                    .groups
                    .token(id)
                    .map_err(|e| self.group_error(e))?,
            ),
            TaskIntrinsic::CancelTaskGroup => Value::Boolean(
                scheduler
                    .groups
                    .cancel(id)
                    .map_err(|e| self.group_error(e))?,
            ),
            TaskIntrinsic::StartTaskInGroup => self.start_group_task(id, &args[1], None)?,
            TaskIntrinsic::StartSupervisedTask => {
                let (Value::Integer(retries), Value::Integer(backoff)) = (&args[2], &args[3])
                else {
                    return Err(self.group_error("RetryLimit and BackoffMillis must be integers"));
                };
                let policy = super::supervision::RetryPolicy::new(*retries, *backoff)
                    .map_err(|e| self.group_error(e))?;
                self.start_group_task(id, &args[1], Some(policy))?
            }
            TaskIntrinsic::CloseTaskGroup | TaskIntrinsic::CloseTaskGroupWithTimeout => {
                let timeout = if operation == TaskIntrinsic::CloseTaskGroupWithTimeout {
                    Some(self.wait_timeout(&args[1])?)
                } else {
                    None
                };
                return self.start_group_close(id, timeout, destination).map(Some);
            }
            TaskIntrinsic::TryCloseCompletedTaskGroup => {
                let completed = scheduler
                    .try_close_completed_group(id, self.task_id)?
                    .map(|failures| self.group_report(failures))
                    .transpose()?;
                completed.map_or(Value::OptionNone, Value::option_some)
            }
            _ => unreachable!("group intrinsic dispatch"),
        };
        Ok(Some(Some(value)))
    }

    fn group_report(&self, failures: Vec<GroupFailure>) -> Result<Value, VmError> {
        let location = SourceLocation::new(1, 1);
        let records = failures
            .into_iter()
            .map(|failure| {
                // Payload-free Pascal enums use their integer backing value at runtime.
                let kind = Value::Integer(failure.kind as i64);
                self.record_value(
                    "Std.Task.TaskFailure",
                    vec![
                        Value::Integer(failure.task as i64),
                        kind,
                        Value::Str(failure.message.into()),
                        Value::Integer(i64::from(failure.code)),
                        Value::Integer(failure.line as i64),
                        Value::Integer(failure.column as i64),
                    ],
                    location,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Array(records.into()))
    }

    /// Report invalid group identities, ownership, worker signatures, or retry bounds.
    pub(in crate::vm::tasks) fn group_error(&self, message: impl Into<String>) -> VmError {
        self.runtime_error(fpas_diagnostics::codes::RUNTIME_INVALID_TASK, message.into(), "Use an open TaskGroup; only its creator may close it, and workers must have immutable captures.")
    }
}
