//! Validation and conversion of borrowed Console register arguments.

use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use crate::vm::VmError;
use crate::vm::hosted::console_cell_records::console_cell_from_value;
use crate::vm::hosted::console_records::console_event_record;
use crate::vm::worker::Worker;

pub(super) fn optional_event(
    worker: &Worker,
    event: Option<fpas_std::ConsoleEvent>,
    location: SourceLocation,
) -> Result<Value, VmError> {
    event.map_or(Ok(Value::OptionNone), |event| {
        Ok(Value::OptionSome(Box::new(console_event_record(
            worker, event, location,
        )?)))
    })
}

pub(super) fn string<'a>(
    arguments: &'a [Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<&'a str, VmError> {
    match value(arguments, index, count, worker)? {
        Value::Str(value) => Ok(value),
        actual => Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Console intrinsic expected string, got {}",
                actual.type_name()
            ),
            "Pass a string value to this Std.Console call.",
        )),
    }
}

pub(super) fn console_cells(
    value: &Value,
    location: SourceLocation,
    worker: &Worker,
) -> Result<Vec<fpas_std::ConsoleCell>, VmError> {
    let Value::Array(values) = value else {
        return Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Expected array of Std.Console.Cell, got {}",
                value.type_name()
            ),
            "Pass an array of Cell values.",
        ));
    };
    values
        .iter()
        .map(|value| console_cell_from_value(value, location))
        .collect()
}

pub(super) fn require_count(
    arguments: &[Value],
    expected: usize,
    worker: &Worker,
) -> Result<(), VmError> {
    if arguments.len() == expected {
        Ok(())
    } else {
        Err(worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Console intrinsic expected {expected} arguments, got {}",
                arguments.len()
            ),
            "Check the compiler intrinsic signature and register argument count.",
        ))
    }
}

pub(super) fn value<'a>(
    arguments: &'a [Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<&'a Value, VmError> {
    require_count(arguments, count, worker)?;
    arguments.get(index).ok_or_else(|| {
        worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Console intrinsic argument is missing",
            "Check the compiler intrinsic signature and register argument count.",
        )
    })
}

pub(super) fn integer(
    arguments: &[Value],
    index: usize,
    count: usize,
    worker: &Worker,
) -> Result<i64, VmError> {
    match value(arguments, index, count, worker)? {
        Value::Integer(value) => Ok(*value),
        actual => Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Console intrinsic expected integer, got {}",
                actual.type_name()
            ),
            "Pass an integer value to this Std.Console call.",
        )),
    }
}
