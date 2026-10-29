//! Hosted dispatch for VM-owned, serialized regular file handles.

mod arguments;
mod call;
mod failure;
mod operations;
mod registry;
mod state;
pub(in crate::execution::vm) mod wait;

#[cfg(test)]
mod pool_tests;
#[cfg(test)]
mod tests;

pub(super) use registry::Files;

use crate::execution::vm::{VmError, worker::Worker};
use crate::{FsIntrinsic, Intrinsic, SourceLocation, Value};
use call::{Call, Output};
use failure::Failure;

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
        let call = Call::prepare(self, operation, values)?;
        let outcome = call.and_then(|call| call.run(&self.hosted, || false));
        self.fs_result(operation, outcome, location)
            .map(|value| Some(Some(value)))
    }

    fn fs_result(
        &self,
        operation: FsIntrinsic,
        outcome: Result<Output, Failure>,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        Ok(match outcome {
            Ok(output) => Value::result_ok(match output {
                Output::Handle(handle) => Value::OpaqueHandle(handle),
                Output::Number(number) => Value::Integer(number),
                Output::Done => Value::Boolean(true),
                Output::Bytes(bytes) => Value::Array(
                    bytes
                        .into_iter()
                        .map(|b| Value::Integer(i64::from(b)))
                        .collect(),
                ),
                Output::Line(line) => line.map_or(Value::OptionNone, |text| {
                    Value::option_some(Value::Str(text.into()))
                }),
            }),
            Err(error) => Value::result_error(error.into_value(
                self.layouts.as_ref(),
                &Intrinsic::Fs(operation).debugger_name(),
                location,
            )?),
        })
    }
}
