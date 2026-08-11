//! Type-aware concretization of textual assignment selectors.

use fpas_bytecode::{DebugType, Executable, Value};

use super::super::inspection::{MutationPath, MutationTarget};
use super::super::types::{DebugErrorKind, DebugSessionError};
use super::model::{DebugAssignmentSelector, DebugAssignmentTarget};

/// Concretizes evaluated textual selectors as the existing writable path model.
pub(in crate::vm::debug) fn target(
    executable: &Executable,
    assignment: &DebugAssignmentTarget,
    mut target: MutationTarget,
    mut current: Value,
    evaluated_indexes: &[Value],
) -> Result<MutationTarget, DebugSessionError> {
    let mut indexes = evaluated_indexes.iter();
    for selector in &assignment.selectors {
        match selector {
            DebugAssignmentSelector::Field(name) => {
                let Value::Record(record) = &current else {
                    return Err(unsupported_path(
                        format!("debug variable target field `{name}` requires a stored record"),
                        "Use a stored record field below a mutable binding.",
                    ));
                };
                let Some(index) = record
                    .body()
                    .layout
                    .fields
                    .iter()
                    .position(|field| field.eq_ignore_ascii_case(name))
                else {
                    return Err(unknown_component(name));
                };
                let expected = record_field_type(
                    executable,
                    target.expected_type,
                    record.body().layout.record,
                    index,
                )?;
                current = record
                    .body()
                    .values
                    .get(index)
                    .cloned()
                    .ok_or_else(path_unavailable)?;
                target.path.push(MutationPath::RecordField(index));
                target.expected_type = expected;
            }
            DebugAssignmentSelector::Index(_) => {
                let index = indexes.next().ok_or_else(path_unavailable)?;
                match &current {
                    Value::Array(values) => {
                        let Value::Integer(index) = index else {
                            return Err(unsupported_path(
                                "debug variable array target requires an Integer index",
                                "Use an Integer expression inside the array brackets.",
                            ));
                        };
                        let index = usize::try_from(*index).map_err(|_| {
                            unknown_index(index.to_string(), "array index is negative or too large")
                        })?;
                        let expected = array_element_type(executable, target.expected_type)?;
                        current = values.get(index).cloned().ok_or_else(|| {
                            unknown_index(index.to_string(), "array index is out of bounds")
                        })?;
                        target.path.push(MutationPath::ArrayIndex(index));
                        target.expected_type = expected;
                    }
                    Value::Dict(entries) => {
                        let expected = dictionary_value_type(executable, target.expected_type)?;
                        let Some((key, value)) = entries
                            .iter()
                            .find(|(candidate, _)| candidate == index)
                            .map(|(key, value)| (key.clone(), value.clone()))
                        else {
                            return Err(unknown_index(
                                index.to_string(),
                                "dictionary key does not already exist",
                            ));
                        };
                        current = value;
                        target.path.push(MutationPath::DictionaryValue(key));
                        target.expected_type = expected;
                    }
                    _ => {
                        return Err(unsupported_path(
                            "debug variable target index requires an array or dictionary",
                            "Use an array element or an existing dictionary key; string editing is unsupported.",
                        ));
                    }
                }
            }
        }
    }
    if indexes.next().is_some() {
        return Err(path_unavailable());
    }
    Ok(target)
}

fn record_field_type(
    executable: &Executable,
    expected: fpas_bytecode::DebugTypeId,
    runtime_record: fpas_bytecode::RecordTypeId,
    field: usize,
) -> Result<fpas_bytecode::DebugTypeId, DebugSessionError> {
    let Some(DebugType::Record(record)) = executable.debug_types.get(expected.get() as usize)
    else {
        return Err(unsupported_metadata());
    };
    if *record != runtime_record {
        return Err(unsupported_metadata());
    }
    executable
        .records
        .get(usize::from(record.get()))
        .and_then(|layout| layout.fields.get(field))
        .map(|field| field.ty)
        .ok_or_else(unsupported_metadata)
}

fn array_element_type(
    executable: &Executable,
    expected: fpas_bytecode::DebugTypeId,
) -> Result<fpas_bytecode::DebugTypeId, DebugSessionError> {
    match executable.debug_types.get(expected.get() as usize) {
        Some(DebugType::Array(element)) => Ok(*element),
        _ => Err(unsupported_metadata()),
    }
}

fn dictionary_value_type(
    executable: &Executable,
    expected: fpas_bytecode::DebugTypeId,
) -> Result<fpas_bytecode::DebugTypeId, DebugSessionError> {
    match executable.debug_types.get(expected.get() as usize) {
        Some(DebugType::Dictionary { value, .. }) => Ok(*value),
        _ => Err(unsupported_metadata()),
    }
}

fn unknown_component(name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!("debug variable target field `{name}` does not exist"),
        hint: "Use a stored field returned by Variables for the current record.".to_string(),
    }
}

fn unknown_index(value: String, detail: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!("debug variable target index `{value}` is unavailable: {detail}"),
        hint: "Use an in-range array index or a dictionary key that already exists.".to_string(),
    }
}

fn unsupported_metadata() -> DebugSessionError {
    unsupported_path(
        "debug variable target lacks assignable portable type metadata",
        "Select a descendant already supported by setVariable.",
    )
}

fn unsupported_path(message: impl Into<String>, hint: impl Into<String>) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: message.into(),
        hint: hint.into(),
    }
}

fn path_unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug variable aggregate path is unavailable".to_string(),
        hint: "Request the stopped state again and retry the textual target.".to_string(),
    }
}
