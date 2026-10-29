//! Typed value admission and results for interactive process handles.

use crate::{Intrinsic, ProcIntrinsic, SourceLocation, Value};

use super::arguments;
use super::failure::Failure;
use super::interactive::MAX_BYTES;
use crate::execution::vm::{VmError, worker::Worker};

enum Output {
    Value(Value),
    Status(Option<std::process::ExitStatus>),
}

impl Worker {
    pub(super) fn execute_interactive_process(
        &self,
        operation: ProcIntrinsic,
        values: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        use ProcIntrinsic::{
            Close, CloseStdin, ReadStderr, ReadStdout, Start, Terminate, TryWait, TryWaitStatus,
            WriteStdin,
        };
        if !matches!(
            operation,
            Start
                | ReadStdout
                | ReadStderr
                | WriteStdin
                | CloseStdin
                | TryWait
                | TryWaitStatus
                | Terminate
                | Close
        ) {
            return Ok(None);
        }
        let processes = &self.hosted.piped_processes;
        let outcome = (|| {
            if operation == Start {
                return arguments::parse_start(values)
                    .and_then(|request| processes.start(&request))
                    .map(|handle| Output::Value(Value::OpaqueHandle(handle)));
            }
            let count = if matches!(operation, ReadStdout | ReadStderr | WriteStdin) {
                2
            } else {
                1
            };
            if values.len() != count {
                return Err(Failure::argument(
                    "Arguments",
                    "Incorrect intrinsic argument count",
                ));
            }
            let Value::OpaqueHandle(handle) = values[0] else {
                return Err(Failure::argument("Process", "Expected a process handle"));
            };
            if operation == Close {
                return processes
                    .close(handle)
                    .map(|()| Output::Value(Value::Boolean(true)));
            }
            if operation == TryWaitStatus {
                return processes
                    .with(handle, |process| process.try_wait())
                    .map(Output::Status);
            }
            processes
                .with(handle, |process| match operation {
                    ReadStdout | ReadStderr => process
                        .read(
                            arguments::integer(&values[1], "MaxBytes")?,
                            operation == ReadStderr,
                        )
                        .map(|bytes| {
                            bytes.map_or(Value::OptionNone, |bytes| {
                                Value::option_some(Value::Array(
                                    bytes
                                        .into_iter()
                                        .map(|b| Value::Integer(i64::from(b)))
                                        .collect(),
                                ))
                            })
                        }),
                    WriteStdin => process
                        .write(&bytes(&values[1])?)
                        .map(|count| Value::Integer(count as i64)),
                    CloseStdin => process.close_stdin().map(Value::Boolean),
                    TryWait => process.try_wait().map(|status| {
                        status.map_or(Value::OptionNone, |status| {
                            Value::option_some(Value::Integer(i64::from(
                                status.code().unwrap_or(-1),
                            )))
                        })
                    }),
                    Terminate => process.terminate().map(|()| Value::Boolean(true)),
                    _ => unreachable!("interactive operation was admitted above"),
                })
                .map(Output::Value)
        })();
        Ok(Some(match outcome {
            Ok(Output::Value(value)) => Value::result_ok(value),
            Ok(Output::Status(status)) => Value::result_ok(match status {
                None => Value::OptionNone,
                Some(status) => Value::option_some(super::status::value(
                    self.layouts.as_ref(),
                    status,
                    location,
                )?),
            }),
            Err(error) => Value::result_error(error.into_value(
                self.layouts.as_ref(),
                &Intrinsic::Proc(operation).debugger_name(),
                arguments::command_context(values),
                false,
                location,
            )?),
        }))
    }
}

pub(super) fn bytes(value: &Value) -> Result<Vec<u8>, Failure> {
    let Value::Array(values) = value else {
        return Err(Failure::argument("Bytes", "Expected an array of integers"));
    };
    if values.len() > MAX_BYTES {
        return Err(Failure::limit("write bytes", MAX_BYTES));
    }
    values
        .iter()
        .map(|value| {
            u8::try_from(arguments::integer(value, "Bytes")?)
                .map_err(|_| Failure::argument("Bytes", "Expected byte values in 0..255"))
        })
        .collect()
}
