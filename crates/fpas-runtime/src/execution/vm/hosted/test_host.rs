//! Hosted screen and input assertions for the VM.

use crate::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

use super::super::VmError;
use super::super::worker::Worker;

impl Worker {
    pub(super) fn execute_test_host_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Test(operation) = intrinsic else {
            return Ok(None);
        };
        if !super::console_io::supported_test(operation) {
            return Ok(None);
        }
        self.execute_test_console_owned(operation, arguments, location)
            .map(Some)
    }

    pub(super) fn test_host_error(&self, message: &str) -> VmError {
        self.runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            message,
            "Check the hosted console state and the assertion arguments.",
        )
    }
}

pub(super) fn integer(
    arguments: &[Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<i64, VmError> {
    match argument(arguments, index, count, worker)? {
        Value::Integer(value) => Ok(*value),
        actual => {
            Err(worker.test_host_error(&format!("Expected integer, got {}", actual.type_name())))
        }
    }
}

pub(super) fn string<'a>(
    arguments: &'a [Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<&'a str, VmError> {
    match argument(arguments, index, count, worker)? {
        Value::Str(value) => Ok(value),
        actual => {
            Err(worker.test_host_error(&format!("Expected string, got {}", actual.type_name())))
        }
    }
}

fn argument<'a>(
    arguments: &'a [Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<&'a Value, VmError> {
    if arguments.len() != count {
        return Err(worker.test_host_error("Hosted test intrinsic argument count mismatch"));
    }
    arguments
        .get(index)
        .ok_or_else(|| worker.test_host_error("Hosted test intrinsic argument is missing"))
}

pub(super) fn argument_count(
    arguments: &[Value],
    count: usize,
    worker: &Worker,
) -> Result<(), VmError> {
    if arguments.len() == count {
        Ok(())
    } else {
        Err(worker.test_host_error("Hosted test intrinsic argument count mismatch"))
    }
}

pub(super) fn coordinate(value: i64, label: &str, worker: &Worker) -> Result<u16, VmError> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            worker.test_host_error(&format!(
                "Screen {label} must be in 1..={}, got {value}",
                u16::MAX
            ))
        })
}
