//! Owned process execution with bounded capture and cooperative supervision.

mod acp;
mod acp_auth;
pub(in crate::execution::vm::hosted) use acp_auth::Authentication as AcpAuthentication;
pub(in crate::execution::vm::hosted) use acp_auth::authenticate as acp_authenticate;
mod acp_output;
mod acp_terminals;
pub(in crate::execution::vm::hosted) use acp_terminals::Terminals as AcpTerminals;
mod arguments;
pub(super) use acp::{AcpLaunch, AcpProcess};
mod engine;
mod failure;
mod interactive;
mod interactive_dispatch;
mod options;
mod pipes;
pub(super) mod platform;
mod registry;
mod run_interactive;
mod status;

#[cfg(test)]
mod tests;

#[cfg(all(test, unix))]
mod failure_tests;

pub(super) use interactive::PipedProcesses;
pub(super) use registry::Processes;

use crate::{Intrinsic, ProcIntrinsic, SourceLocation, Value};

use super::super::worker::Worker;
use super::super::{Vm, VmError};

impl Worker {
    pub(super) fn execute_proc_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Proc(operation) = intrinsic else {
            return Ok(None);
        };
        if operation == ProcIntrinsic::RunInteractive {
            return self
                .execute_run_interactive(arguments, location)
                .map(|value| Some(Some(value)));
        }
        if let Some(value) = self.execute_interactive_process(operation, arguments, location)? {
            return Ok(Some(Some(value)));
        }
        let options = matches!(
            operation,
            ProcIntrinsic::RunWithOptions | ProcIntrinsic::RunCaptureWithOptions
        );
        let capture = matches!(
            operation,
            ProcIntrinsic::RunCapture | ProcIntrinsic::RunCaptureWithOptions
        );
        if !matches!(
            operation,
            ProcIntrinsic::Run
                | ProcIntrinsic::RunCapture
                | ProcIntrinsic::RunWithOptions
                | ProcIntrinsic::RunCaptureWithOptions
        ) {
            return Ok(None);
        }
        let name = intrinsic.debugger_name();
        let request = arguments::parse(arguments, options, capture);
        let command = arguments::command_context(arguments);
        let result = request.and_then(|request| {
            if let Some(token) = request.options.token {
                self.hosted
                    .cancellations
                    .is_cancelled(token)
                    .map_err(|error| failure::Failure::argument("CancellationToken", error))?;
            }
            let receiver = if !capture {
                self.hosted
                    .program_stderr
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .clone()
            } else {
                None
            };
            self.hosted.processes.run(&request, receiver, || {
                request.options.token.is_some_and(|token| {
                    self.hosted
                        .cancellations
                        .is_cancelled(token)
                        .unwrap_or(true)
                })
            })
        });
        let value = match result {
            Ok(output) if capture => Value::result_ok(self.record_value(
                "Std.Proc.ProcessOutput",
                vec![
                    Value::Integer(output.code),
                    Value::Str(output.stdout.into()),
                    Value::Str(output.stderr.into()),
                ],
                location,
            )?),
            Ok(output) => Value::result_ok(Value::Integer(output.code)),
            Err(error) => Value::result_error(error.into_value(
                self.layouts.as_ref(),
                &name,
                command,
                !options,
                location,
            )?),
        };
        Ok(Some(Some(value)))
    }
}

impl Vm {
    /// Route stderr lines of legacy `Std.Proc.Run` children to the installed receiver.
    pub fn set_program_stderr(&mut self, receiver: crate::stdlib::ProgramStderr) {
        *self
            .hosted
            .program_stderr
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(receiver);
    }
}
