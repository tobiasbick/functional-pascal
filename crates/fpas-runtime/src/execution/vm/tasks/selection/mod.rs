//! Typed mixed-source selection and task-owned single-use cases.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

#[cfg(test)]
mod ownership_tests;
mod polling;
mod registry;

use super::{TaskClock, TaskSuspensionState};
use crate::execution::vm::{VmError, worker::Worker};
use crate::{Intrinsic, Register, TaskIntrinsic, Value};
pub(in crate::execution::vm) use registry::{CaseRegistry, CaseSource};
use registry::{MAX_CASES, WaitCase};
use std::time::{Duration, Instant};

/// Claimed cases retained while normal or debugger execution waits for a winner.
pub(in crate::execution::vm) struct SelectionWait {
    cases: Vec<WaitCase>,
    started: Instant,
    debug_started: u64,
    destination: Option<Register>,
}

impl SelectionWait {
    /// Expose the earliest timer to the debugger without changing the selection origin.
    pub(super) fn debug_state(&self, clock: &TaskClock) -> TaskSuspensionState {
        self.cases
            .iter()
            .filter_map(|case| match case.source {
                CaseSource::Timer(milliseconds) => Some(
                    milliseconds
                        .saturating_sub(clock.now_millis().saturating_sub(self.debug_started)),
                ),
                _ => None,
            })
            .min()
            .map_or(TaskSuspensionState::Waiting, |milliseconds| {
                TaskSuspensionState::Sleeping {
                    remaining: Duration::from_millis(milliseconds),
                }
            })
    }
}

impl crate::TraceRoots for SelectionWait {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), crate::OwnershipError> {
        for case in &self.cases {
            case.trace_roots(visit)?;
        }
        Ok(())
    }
}

impl Worker {
    /// Route case construction, explicit close, and mixed-source selection.
    pub(in crate::execution::vm::tasks) fn selection_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        args: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Task(operation) = intrinsic else {
            return Ok(None);
        };
        if operation == TaskIntrinsic::Select {
            let [Value::Array(values)] = args else {
                return Err(self.selection_error("Select expects an array of WaitCase handles"));
            };
            if !(1..=MAX_CASES).contains(&values.len()) {
                return Err(
                    self.selection_error("Select requires between 1 and 1024 WaitCase handles")
                );
            }
            let ids = values
                .iter()
                .map(|value| self.selection_handle(value))
                .collect::<Result<Vec<_>, _>>()?;
            let cases = self
                .hosted
                .cases
                .claim(self.task_id, &ids)
                .map_err(|e| self.selection_error(e))?;
            let wait = SelectionWait {
                cases,
                started: Instant::now(),
                debug_started: if self.debug_tasks {
                    self.task_clock_ref().now_millis()
                } else {
                    0
                },
                destination,
            };
            self.run_selection(wait)?;
            return Ok(Some(None));
        }
        if operation == TaskIntrinsic::CloseWaitCase {
            let [value] = args else {
                return Err(self.selection_error("CloseWaitCase expects one WaitCase handle"));
            };
            let id = self.selection_handle(value)?;
            let closed = self
                .hosted
                .cases
                .close(self.task_id, id)
                .map_err(|e| self.selection_error(e))?;
            return Ok(Some(Some(Value::Boolean(closed))));
        }
        let count = match operation {
            TaskIntrinsic::SendCase => 3,
            TaskIntrinsic::ReceiveCase
            | TaskIntrinsic::TaskCase
            | TaskIntrinsic::TimerCase
            | TaskIntrinsic::CancellationCase => 2,
            _ => return Ok(None),
        };
        if args.len() != count {
            return Err(self.selection_error(format!(
                "Selection case constructor expects {count} arguments"
            )));
        }
        let arity = usize::from(matches!(
            operation,
            TaskIntrinsic::SendCase | TaskIntrinsic::ReceiveCase
        ));
        if operation == TaskIntrinsic::SendCase {
            self.require_transferable_values(&args[1..2])?;
        }
        self.require_storable_value(&args[count - 1])?;
        let callback = self.validate_callback(&args[count - 1], arity)?;
        let identity = match operation {
            TaskIntrinsic::TaskCase => {
                let Value::Task(id) = args[0] else {
                    return Err(self.task_type_error("task", &args[0]));
                };
                self.validate_selection_tasks(&[id])?;
                id
            }
            TaskIntrinsic::TimerCase => {
                let Value::Integer(milliseconds) = args[0] else {
                    return Err(self.task_type_error("integer", &args[0]));
                };
                u64::try_from(milliseconds).map_err(|_| {
                    self.selection_error("TimerCase requires non-negative milliseconds")
                })?
            }
            TaskIntrinsic::CancellationCase => {
                let id = self.selection_handle(&args[0])?;
                self.hosted
                    .cancellations
                    .is_cancelled(id)
                    .map_err(|e| self.selection_error(e))?;
                id
            }
            _ => {
                let id = self.selection_handle(&args[0])?;
                self.hosted
                    .channels
                    .validate(id)
                    .map_err(|e| self.selection_error(e))?;
                id
            }
        };
        let source = match operation {
            TaskIntrinsic::SendCase => CaseSource::Send(identity, args[1].clone()),
            TaskIntrinsic::ReceiveCase => CaseSource::Receive(identity),
            TaskIntrinsic::TaskCase => CaseSource::Task(identity),
            TaskIntrinsic::TimerCase => CaseSource::Timer(identity),
            TaskIntrinsic::CancellationCase => CaseSource::Cancellation(identity),
            _ => unreachable!("case constructor dispatch"),
        };
        self.register_wait_case(source, callback.clone())
            .map(|value| Some(Some(value)))
    }

    pub(in crate::execution::vm) fn register_wait_case(
        &self,
        source: CaseSource,
        callback: crate::SharedFunction,
    ) -> Result<Value, VmError> {
        let id = self
            .hosted
            .cases
            .create_in_domain(self.task_id, &self.domain, || WaitCase {
                roots: None,
                callback,
                source,
            })
            .map_err(|e| self.selection_error(e))?;
        Ok(Value::OpaqueHandle(id))
    }

    fn selection_handle(&self, value: &Value) -> Result<u64, VmError> {
        match value {
            Value::OpaqueHandle(id) => Ok(*id),
            _ => Err(self.task_type_error("opaque resource handle", value)),
        }
    }

    pub(in crate::execution::vm) fn selection_error(&self, message: impl Into<String>) -> VmError {
        self.runtime_error(fpas_diagnostics::codes::RUNTIME_INVALID_TASK, message.into(), "Use live WaitCase handles on their creating task and match each callback to its source.")
    }
}
