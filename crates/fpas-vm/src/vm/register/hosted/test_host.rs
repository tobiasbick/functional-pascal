//! Hosted screen and input assertions for the register VM.

use fpas_bytecode::{Intrinsic, SourceLocation, TestIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::{assert_screen_cell, assert_screen_line};

use super::super::VmError;
use super::super::worker::RegisterWorker;

impl RegisterWorker {
    pub(super) fn execute_test_host_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Test(operation) = intrinsic else {
            return Ok(None);
        };
        match operation {
            TestIntrinsic::AssertScreenLine => {
                let expected = string(arguments, 0, 2, self)?;
                let row = coordinate(integer(arguments, 1, 2, self)?, "row", self)?;
                let actual = self.with_console(|console| console.query_screen_line(row));
                assert_screen_line(expected.to_owned(), actual, location)?;
            }
            TestIntrinsic::AssertScreenCell => {
                let column = coordinate(integer(arguments, 0, 5, self)?, "column", self)?;
                let row = coordinate(integer(arguments, 1, 5, self)?, "row", self)?;
                let expected = string(arguments, 2, 5, self)?;
                let mut chars = expected.chars();
                let expected = match (chars.next(), chars.next()) {
                    (Some(character), None) => character,
                    _ => {
                        return Err(self.test_host_error("AssertScreenCell expected one character"));
                    }
                };
                let foreground = integer(arguments, 3, 5, self)?;
                let background = integer(arguments, 4, 5, self)?;
                let (actual, actual_foreground, actual_background) = self
                    .with_console(|console| console.query_screen_cell(column, row))
                    .ok_or_else(|| {
                        self.test_host_error("AssertScreenCell coordinate is outside the screen")
                    })?;
                assert_screen_cell(
                    expected,
                    foreground,
                    background,
                    actual,
                    actual_foreground,
                    actual_background,
                    location,
                )?;
            }
            TestIntrinsic::PushReadLn => {
                let text = string(arguments, 0, 1, self)?;
                self.with_text_input(|input| input.push_line(text));
            }
            _ => return Ok(None),
        }
        Ok(Some(None))
    }

    fn test_host_error(&self, message: &str) -> VmError {
        self.runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            message,
            "Check the hosted console state and the assertion arguments.",
        )
    }
}

fn integer(
    arguments: &[Value],
    index: usize,
    count: usize,
    worker: &RegisterWorker,
) -> Result<i64, VmError> {
    match argument(arguments, index, count, worker)? {
        Value::Integer(value) => Ok(*value),
        actual => {
            Err(worker.test_host_error(&format!("Expected integer, got {}", actual.type_name())))
        }
    }
}

fn string<'a>(
    arguments: &'a [Value],
    index: usize,
    count: usize,
    worker: &RegisterWorker,
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
    worker: &RegisterWorker,
) -> Result<&'a Value, VmError> {
    if arguments.len() != count {
        return Err(worker.test_host_error("Hosted test intrinsic argument count mismatch"));
    }
    arguments
        .get(index)
        .ok_or_else(|| worker.test_host_error("Hosted test intrinsic argument is missing"))
}

fn coordinate(value: i64, label: &str, worker: &RegisterWorker) -> Result<u16, VmError> {
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
