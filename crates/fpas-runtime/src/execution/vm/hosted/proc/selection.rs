//! Selectable pipe operations commit only when their case wins ordered probing.

#[cfg(test)]
mod tests;

use super::{failure::Failure, interactive::Process};
use crate::execution::vm::{VmError, tasks::selection::CaseSource, worker::Worker};
use crate::{Intrinsic, ProcIntrinsic, Value};

pub(in crate::execution::vm) enum ProcessCase {
    ReadStdout(i64),
    ReadStderr(i64),
    WriteStdin(crate::SharedBytes),
    CloseStdin,
    WaitStatus,
}

enum Output {
    Bytes(Vec<u8>),
    Count(usize),
    Closed,
    Status(std::process::ExitStatus),
}

impl ProcessCase {
    fn operation(&self) -> ProcIntrinsic {
        match self {
            Self::ReadStdout(_) => ProcIntrinsic::ReadStdoutCase,
            Self::ReadStderr(_) => ProcIntrinsic::ReadStderrCase,
            Self::WriteStdin(_) => ProcIntrinsic::WriteStdinCase,
            Self::CloseStdin => ProcIntrinsic::CloseStdinCase,
            Self::WaitStatus => ProcIntrinsic::WaitStatusCase,
        }
    }

    fn probe(&self, process: &mut Process) -> Result<Option<Output>, Failure> {
        Ok(match self {
            Self::ReadStdout(maximum) | Self::ReadStderr(maximum) => process
                .read(*maximum, matches!(self, Self::ReadStderr(_)))?
                .map(Output::Bytes),
            Self::WriteStdin(bytes) => {
                let count = process.write(bytes)?;
                (count > 0 || bytes.is_empty()).then_some(Output::Count(count))
            }
            Self::CloseStdin => process.close_stdin()?.then_some(Output::Closed),
            Self::WaitStatus => process.try_wait()?.map(Output::Status),
        })
    }
}

impl Worker {
    pub(super) fn execute_process_case(
        &self,
        operation: ProcIntrinsic,
        arguments: &[Value],
    ) -> Result<Option<Value>, VmError> {
        use ProcIntrinsic::{
            CloseStdinCase, ReadStderrCase, ReadStdoutCase, WaitStatusCase, WriteStdinCase,
        };
        let count = match operation {
            ReadStdoutCase | ReadStderrCase | WriteStdinCase => 3,
            CloseStdinCase | WaitStatusCase => 2,
            _ => return Ok(None),
        };
        if arguments.len() != count {
            return Err(self.selection_error("Incorrect process case argument count"));
        }
        let Value::OpaqueHandle(handle) = arguments[0] else {
            return Err(self.selection_error("Expected a process handle"));
        };
        let callback = &arguments[count - 1];
        self.require_storable_value(callback)?;
        let callback = self.validate_callback(callback, 1)?.clone();
        let source = match operation {
            ReadStdoutCase | ReadStderrCase => {
                let Value::Integer(maximum) = arguments[1] else {
                    return Err(self.selection_error("MaxBytes must be an integer"));
                };
                if !(1..=super::interactive::MAX_BYTES as i64).contains(&maximum) {
                    return Err(self.selection_error("MaxBytes must be in 1..1048576"));
                }
                if operation == ReadStdoutCase {
                    ProcessCase::ReadStdout(maximum)
                } else {
                    ProcessCase::ReadStderr(maximum)
                }
            }
            WriteStdinCase => ProcessCase::WriteStdin(
                super::interactive_dispatch::bytes(&arguments[1]).map_err(|_| {
                    self.selection_error(
                        "Bytes must be a Std.Bytes.ByteBuffer of at most 1048576 bytes",
                    )
                })?,
            ),
            CloseStdinCase => ProcessCase::CloseStdin,
            WaitStatusCase => ProcessCase::WaitStatus,
            _ => unreachable!("process case dispatch"),
        };
        self.register_wait_case(CaseSource::Process(handle, source), callback)
            .map(Some)
    }

    pub(in crate::execution::vm) fn poll_process_case(
        &self,
        handle: u64,
        operation: &ProcessCase,
    ) -> Result<Option<Value>, VmError> {
        // Build VM values only after releasing the per-process mutex.
        let output = self
            .hosted
            .piped_processes
            .try_with(handle, |process| operation.probe(process));
        let location = self.intrinsic_location();
        let outcome = match output {
            Ok(None) => return Ok(None),
            Ok(Some(output)) => Value::result_ok(match output {
                Output::Bytes(bytes) => Value::native_bytes(bytes),
                Output::Count(count) => Value::Integer(count as i64),
                Output::Closed => Value::Boolean(true),
                Output::Status(status) => {
                    super::status::value(self.layouts.as_ref(), status, location)?
                }
            }),
            Err(error) => Value::result_error(error.into_value(
                self.layouts.as_ref(),
                &Intrinsic::Proc(operation.operation()).debugger_name(),
                &handle.to_string(),
                false,
                location,
            )?),
        };
        Ok(Some(outcome))
    }
}
