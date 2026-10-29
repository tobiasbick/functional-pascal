//! Task-owned process state; native polling does not retain FPAS frames or values.

use super::{arguments, engine::Execution, failure::Failure};
use crate::execution::vm::{VmError, tasks::TaskSuspension, worker::Worker};
use crate::{ProcIntrinsic as Op, Register, SourceLocation, Value};
use std::task::Poll;

pub(in crate::execution::vm) struct Wait {
    execution: Execution,
    input: String,
    token: Option<u64>,
    operation: Op,
    command: String,
    destination: Option<Register>,
    location: SourceLocation,
}

impl Wait {
    /// Productive pipe probes yield through the ready queue; idle probes use the timer.
    pub(in crate::execution::vm) fn made_progress(&self) -> bool {
        self.execution.made_progress()
    }
}

impl Worker {
    pub(in crate::execution::vm) fn start_proc_wait(
        &mut self,
        operation: Op,
        values: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Value>, VmError> {
        let options = matches!(operation, Op::RunWithOptions | Op::RunCaptureWithOptions);
        let capture = matches!(operation, Op::RunCapture | Op::RunCaptureWithOptions);
        let location = self.intrinsic_location();
        let request = arguments::parse(values, options, capture);
        let command = arguments::command_context(values);
        let prepared = request.and_then(|request| {
            if let Some(token) = request.options.token {
                self.hosted
                    .cancellations
                    .is_cancelled(token)
                    .map_err(|error| Failure::argument("CancellationToken", error))?;
            }
            let receiver = if capture {
                None
            } else {
                self.hosted
                    .program_stderr
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone()
            };
            let execution = self.hosted.processes.start(&request, receiver, || {
                request.options.token.is_some_and(|token| {
                    self.hosted
                        .cancellations
                        .is_cancelled(token)
                        .unwrap_or(true)
                })
            })?;
            Ok(Wait {
                execution,
                input: request.options.input.to_owned(),
                token: request.options.token,
                operation,
                command: command.to_owned(),
                destination,
                location,
            })
        });
        let wait = match prepared {
            Ok(wait) => wait,
            Err(error) => {
                return self
                    .proc_result(operation, command, Err(error), location)
                    .map(Some);
            }
        };
        self.task_suspension = Some(TaskSuspension::Proc(Box::new(wait)));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(None)
    }

    pub(in crate::execution::vm) fn poll_proc_wait(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        let outcome = wait.execution.poll(wait.input.as_bytes(), || {
            wait.token.is_some_and(|token| {
                self.hosted
                    .cancellations
                    .is_cancelled(token)
                    .unwrap_or(true)
            })
        });
        match outcome {
            Poll::Pending => {
                self.task_suspension = Some(TaskSuspension::Proc(Box::new(wait)));
                Ok(false)
            }
            Poll::Ready(outcome) => {
                let value =
                    self.proc_result(wait.operation, &wait.command, outcome, wait.location)?;
                if let Some(destination) = wait.destination {
                    self.write(destination, value)?;
                }
                Ok(true)
            }
        }
    }
}
