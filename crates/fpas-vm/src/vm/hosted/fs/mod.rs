//! Hosted dispatch for VM-owned, serialized regular file handles.

mod arguments;
mod failure;
mod operations;
mod registry;
mod state;

#[cfg(test)]
mod tests;

pub(super) use registry::Files;

use fpas_bytecode::{FsIntrinsic, Intrinsic, SourceLocation, Value};

use self::failure::Failure;
use crate::vm::{VmError, worker::Worker};

impl Worker {
    pub(super) fn execute_fs_intrinsic(
        &self,
        intrinsic: Intrinsic,
        values: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Fs(operation) = intrinsic else {
            return Ok(None);
        };
        let result = |outcome: Result<Value, Failure>| -> Result<Value, VmError> {
            Ok(match outcome {
                Ok(value) => Value::result_ok(value),
                Err(error) => Value::result_error(error.into_value(
                    self.layouts.as_ref(),
                    &intrinsic.debugger_name(),
                    location,
                )?),
            })
        };
        let files = &self.hosted.files;
        let value = match operation {
            FsIntrinsic::Open => {
                arguments::count(self, values, 2)?;
                let path = arguments::text(self, &values[0])?;
                result(
                    arguments::ordinal(&values[1], true)
                        .and_then(|mode| files.open(path, mode))
                        .map(Value::OpaqueHandle),
                )?
            }
            FsIntrinsic::Close | FsIntrinsic::Tell | FsIntrinsic::Flush => {
                arguments::count(self, values, 1)?;
                let handle = arguments::handle(self, &values[0], "File")?;
                result(match operation {
                    FsIntrinsic::Close => files.close(handle).map(|()| Value::Boolean(true)),
                    FsIntrinsic::Tell => files.tell(handle).map(Value::Integer),
                    _ => files.flush(handle).map(|()| Value::Boolean(true)),
                })?
            }
            FsIntrinsic::Seek => {
                arguments::count(self, values, 3)?;
                let handle = arguments::handle(self, &values[0], "File")?;
                let offset = arguments::integer(self, &values[1], "Offset")?;
                result(
                    arguments::ordinal(&values[2], false)
                        .and_then(|origin| files.seek(handle, offset, origin))
                        .map(Value::Integer),
                )?
            }
            FsIntrinsic::Read
            | FsIntrinsic::ReadWithCancellation
            | FsIntrinsic::Write
            | FsIntrinsic::WriteWithCancellation
            | FsIntrinsic::ReadLine
            | FsIntrinsic::ReadLineWithCancellation => {
                let cancellable = matches!(
                    operation,
                    FsIntrinsic::ReadWithCancellation
                        | FsIntrinsic::WriteWithCancellation
                        | FsIntrinsic::ReadLineWithCancellation
                );
                let line = matches!(
                    operation,
                    FsIntrinsic::ReadLine | FsIntrinsic::ReadLineWithCancellation
                );
                arguments::count(
                    self,
                    values,
                    usize::from(!line) + 1 + usize::from(cancellable),
                )?;
                let handle = arguments::handle(self, &values[0], "File")?;
                let token = if cancellable {
                    Some(arguments::handle(self, &values[values.len() - 1], "Token")?)
                } else {
                    None
                };
                // A cancelled token is still fully validated before any operation observes it.
                let valid_token = token.map_or(Ok(false), |token| {
                    self.hosted
                        .cancellations
                        .is_cancelled(token)
                        .map_err(|error| Failure::argument("Token", error))
                });
                let cancelled = || {
                    token.is_some_and(|token| {
                        self.hosted
                            .cancellations
                            .is_cancelled(token)
                            .unwrap_or(true)
                    })
                };
                let outcome = match operation {
                    FsIntrinsic::Read | FsIntrinsic::ReadWithCancellation => {
                        let size = arguments::integer(self, &values[1], "MaxBytes")?;
                        valid_token
                            .and_then(|_| files.read(handle, size, cancelled))
                            .map(|bytes| {
                                Value::Array(
                                    bytes
                                        .into_iter()
                                        .map(|byte| Value::Integer(i64::from(byte)))
                                        .collect(),
                                )
                            })
                    }
                    FsIntrinsic::Write | FsIntrinsic::WriteWithCancellation => {
                        arguments::bytes(self, &values[1])?
                            .and_then(|bytes| {
                                valid_token.and_then(|_| files.write(handle, &bytes, cancelled))
                            })
                            .map(|count| Value::Integer(count as i64))
                    }
                    _ => valid_token
                        .and_then(|_| files.read_line(handle, cancelled))
                        .map(|line| {
                            line.map_or(Value::OptionNone, |text| {
                                Value::option_some(Value::Str(text.into()))
                            })
                        }),
                };
                result(outcome)?
            }
            _ => return Ok(None),
        };
        Ok(Some(Some(value)))
    }
}
