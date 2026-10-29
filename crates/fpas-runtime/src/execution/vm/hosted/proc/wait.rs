//! Native process jobs retain no FPAS frames and include launch and cleanup.

use super::{
    arguments,
    failure::{Failure, Kind},
    job::{Job, Operation, Output},
    owned::OwnedRequest,
};
use crate::execution::vm::{VmError, tasks::TaskSuspension, worker::Worker};
use crate::{Intrinsic, ProcIntrinsic as Op, Register, SourceLocation, Value};
use std::sync::Arc;

pub(in crate::execution::vm) struct Wait {
    state: State,
    operation: Op,
    command: String,
    destination: Option<Register>,
    location: SourceLocation,
}

enum State {
    Running(Job),
    QueuedClose(u64),
}

impl Worker {
    pub(in crate::execution::vm) fn start_proc_wait(
        &mut self,
        operation: Op,
        values: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Value>, VmError> {
        let location = self.intrinsic_location();
        let command = arguments::command_context(values).to_owned();
        let prepared = (|| {
            let request = if operation == Op::Close {
                if values.len() != 1 {
                    return Err(Failure::argument(
                        "Arguments",
                        "Expected one process handle",
                    ));
                }
                let Value::OpaqueHandle(handle) = values[0] else {
                    return Err(Failure::argument("Process", "Expected a process handle"));
                };
                if self
                    .hosted
                    .process_jobs
                    .load(std::sync::atomic::Ordering::Acquire)
                    >= super::job::LIMIT
                {
                    return Ok(State::QueuedClose(handle));
                }
                return match Job::start(Arc::clone(&self.hosted), Operation::Close(handle)) {
                    Ok(job) => Ok(State::Running(job)),
                    Err(Failure {
                        kind: Kind::Limit("native process jobs", _),
                        ..
                    }) => Ok(State::QueuedClose(handle)),
                    Err(error) => Err(error),
                };
            } else if operation == Op::Start {
                arguments::parse_start(values)?
            } else if operation == Op::RunInteractive {
                arguments::parse_interactive(values)?
            } else {
                arguments::parse(
                    values,
                    matches!(operation, Op::RunWithOptions | Op::RunCaptureWithOptions),
                    matches!(operation, Op::RunCapture | Op::RunCaptureWithOptions),
                )?
            };
            if let Some(token) = request.options.token {
                self.hosted
                    .cancellations
                    .is_cancelled(token)
                    .map_err(|error| Failure::argument("CancellationToken", error))?;
            }
            let request = OwnedRequest::new(&request);
            let request = match operation {
                Op::Start => Operation::Start(request),
                Op::RunInteractive => Operation::Interactive(request),
                _ => Operation::Run(request),
            };
            Job::start(Arc::clone(&self.hosted), request).map(State::Running)
        })();
        let state = match prepared {
            Ok(state) => state,
            Err(error) => {
                return self
                    .native_proc_result(operation, &command, Err(error), location)
                    .map(Some);
            }
        };
        self.task_suspension = Some(TaskSuspension::Proc(Box::new(Wait {
            state,
            operation,
            command,
            destination,
            location,
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(None)
    }

    pub(in crate::execution::vm) fn poll_proc_wait(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        if let State::QueuedClose(handle) = wait.state {
            match Job::start(Arc::clone(&self.hosted), Operation::Close(handle)) {
                Ok(job) => wait.state = State::Running(job),
                Err(Failure {
                    kind: Kind::Limit("native process jobs", _),
                    ..
                }) => {
                    self.task_suspension = Some(TaskSuspension::Proc(Box::new(wait)));
                    return Ok(false);
                }
                Err(error) => {
                    let value = self.native_proc_result(
                        wait.operation,
                        &wait.command,
                        Err(error),
                        wait.location,
                    )?;
                    if let Some(destination) = wait.destination {
                        self.write(destination, value)?;
                    }
                    return Ok(true);
                }
            }
        }
        let State::Running(job) = &wait.state else {
            unreachable!("queued process close was handled")
        };
        let Some(outcome) = job.poll() else {
            self.task_suspension = Some(TaskSuspension::Proc(Box::new(wait)));
            return Ok(false);
        };
        let value =
            self.native_proc_result(wait.operation, &wait.command, outcome, wait.location)?;
        if let Some(destination) = wait.destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }

    fn native_proc_result(
        &self,
        operation: Op,
        command: &str,
        result: Result<Output, Failure>,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        if matches!(
            operation,
            Op::Run | Op::RunCapture | Op::RunWithOptions | Op::RunCaptureWithOptions
        ) {
            return self.proc_result(
                operation,
                command,
                result.map(|output| match output {
                    Output::Run(output) => output,
                    _ => unreachable!("native process output matches its operation"),
                }),
                location,
            );
        }
        Ok(match result {
            Ok(Output::Started(handle)) => Value::result_ok(Value::OpaqueHandle(handle)),
            Ok(Output::Interactive(status)) => Value::result_ok(super::status::value(
                self.layouts.as_ref(),
                status,
                location,
            )?),
            Ok(Output::Closed) => Value::result_ok(Value::Boolean(true)),
            Ok(Output::Run(_)) => unreachable!("native process output matches its operation"),
            Err(error) => Value::result_error(error.into_value(
                self.layouts.as_ref(),
                &Intrinsic::Proc(operation).debugger_name(),
                command,
                false,
                location,
            )?),
        })
    }
}
