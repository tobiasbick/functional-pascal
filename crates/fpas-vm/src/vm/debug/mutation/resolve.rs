//! Type-aware concretization of textual assignment selectors.

use fpas_bytecode::{DebugType, Executable, Value};

use super::super::inspection::{MutationPath, MutationTarget, PayloadError, resolve_payload};
use super::super::types::{DebugErrorKind, DebugSessionError};
use super::model::{DebugAssignmentSelector, DebugAssignmentTarget};
use super::transition::{self, QualifiedSuffix, SuffixResolution, TransitionSpec};

/// Concretized textual assignment: an existing path or a complete variant switch.
#[derive(Clone)]
pub(in crate::vm::debug) enum ResolvedAssignment {
    /// Ordinary writable descendant of the live value.
    Existing {
        /// Concretized mutation target.
        target: MutationTarget,
        /// Current materialized value at that path.
        current: Value,
    },
    /// Inactive single-payload variant that replaces the nearest wrapper.
    Transition {
        /// Writable path of the wrapper to replace.
        target: MutationTarget,
        /// Complete variant to construct from the evaluated payload.
        spec: TransitionSpec,
    },
}

/// Concretizes evaluated textual selectors as the existing writable path model.
pub(in crate::vm::debug) fn target_with_value(
    executable: &Executable,
    assignment: &DebugAssignmentTarget,
    target: MutationTarget,
    current: Value,
    evaluated_indexes: &[Value],
) -> Result<(MutationTarget, Value), DebugSessionError> {
    match resolve_assignment(executable, assignment, target, current, evaluated_indexes)? {
        ResolvedAssignment::Existing { target, current } => Ok((target, current)),
        ResolvedAssignment::Transition { .. } => Err(unsupported_path(
            "debug variable target uses a variant transition that this operation does not support",
            "Use expression.set or setExpression with a qualified single-payload target such as `Some.value`.",
        )),
    }
}

/// Concretizes a textual target, including qualified inactive-variant suffixes.
pub(in crate::vm::debug) fn resolve_assignment(
    executable: &Executable,
    assignment: &DebugAssignmentTarget,
    mut target: MutationTarget,
    mut current: Value,
    evaluated_indexes: &[Value],
) -> Result<ResolvedAssignment, DebugSessionError> {
    let mut indexes = evaluated_indexes.iter();
    let mut index = 0;
    while index < assignment.selectors.len() {
        match &assignment.selectors[index] {
            DebugAssignmentSelector::Field(name) => {
                if let Value::Record(record) = &current {
                    let Some(field_index) = record
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
                        field_index,
                    )?;
                    current = record
                        .body()
                        .values
                        .get(field_index)
                        .cloned()
                        .ok_or_else(path_unavailable)?;
                    target.path.push(MutationPath::RecordField(field_index));
                    target.expected_type = expected;
                    index += 1;
                    continue;
                }
                let suffix = transition::resolve_suffix(
                    executable,
                    &current,
                    target.expected_type,
                    &assignment.selectors[index..],
                )?;
                let unmatched = match suffix {
                    Some(SuffixResolution::Exact(qualified)) => match qualified {
                        QualifiedSuffix::ActivePayload {
                            component,
                            expected,
                            consumed,
                        } => {
                            current =
                                super::replace::resolve(&current, std::slice::from_ref(&component))
                                    .cloned()
                                    .ok_or_else(path_unavailable)?;
                            target.path.push(component);
                            target.expected_type = expected;
                            index += consumed;
                            continue;
                        }
                        QualifiedSuffix::Switch(spec) => {
                            if indexes.next().is_some() {
                                return Err(path_unavailable());
                            }
                            return Ok(ResolvedAssignment::Transition { target, spec });
                        }
                    },
                    Some(SuffixResolution::Unmatched(error)) => Some(error),
                    None => None,
                };
                match resolve_payload(executable, target.expected_type, &current, name) {
                    Ok((component, expected)) => {
                        current =
                            super::replace::resolve(&current, std::slice::from_ref(&component))
                                .cloned()
                                .ok_or_else(path_unavailable)?;
                        target.path.push(component);
                        target.expected_type = expected;
                        index += 1;
                    }
                    Err(payload_error) => {
                        return Err(
                            unmatched.unwrap_or_else(|| map_payload_error(name, payload_error))
                        );
                    }
                }
            }
            DebugAssignmentSelector::Index(_) => {
                let evaluated = indexes.next().ok_or_else(path_unavailable)?;
                match &current {
                    Value::Array(values) => {
                        let Value::Integer(evaluated) = evaluated else {
                            return Err(unsupported_path(
                                "debug variable array target requires an Integer index",
                                "Use an Integer expression inside the array brackets.",
                            ));
                        };
                        let array_index = usize::try_from(*evaluated).map_err(|_| {
                            unknown_index(
                                evaluated.to_string(),
                                "array index is negative or too large",
                            )
                        })?;
                        let expected = array_element_type(executable, target.expected_type)?;
                        current = values.get(array_index).cloned().ok_or_else(|| {
                            unknown_index(array_index.to_string(), "array index is out of bounds")
                        })?;
                        target.path.push(MutationPath::ArrayIndex(array_index));
                        target.expected_type = expected;
                    }
                    Value::Dict(entries) => {
                        let expected = dictionary_value_type(executable, target.expected_type)?;
                        let Some((key, value)) = entries
                            .iter()
                            .find(|(candidate, _)| candidate == evaluated)
                            .map(|(key, value)| (key.clone(), value.clone()))
                        else {
                            return Err(unknown_index(
                                evaluated.to_string(),
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
                index += 1;
            }
        }
    }
    if indexes.next().is_some() {
        return Err(path_unavailable());
    }
    Ok(ResolvedAssignment::Existing { target, current })
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

fn map_payload_error(name: &str, error: PayloadError) -> DebugSessionError {
    match error {
        PayloadError::UnknownField { active } => unknown_payload_field(name, &active),
        PayloadError::UnsupportedActive { active } => unsupported_payload(name, &active),
        PayloadError::UnavailableMetadata { detail } => unsupported_path(
            format!(
                "debug variable target field `{name}` lacks assignable portable type metadata: {detail}"
            ),
            "Select a payload child already returned as writable by Variables, or replace the complete binding with a constructor.",
        ),
    }
}

fn unknown_component(name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!("debug variable target field `{name}` does not exist"),
        hint: "Use a stored field returned by Variables for the current record.".to_string(),
    }
}

fn unknown_payload_field(name: &str, active: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!(
            "debug variable target field `{name}` does not exist on active `{active}`"
        ),
        hint: format!(
            "Use a payload field returned by Variables for `{active}`, such as a declared enum field or `.value`, or a qualified variant such as `Count.Value`."
        ),
    }
}

fn unsupported_payload(name: &str, active: &str) -> DebugSessionError {
    let hint = if active == "Option.None" {
        "Assign `Some.value` to construct `Option.Some`, or replace the complete binding with `Some(...)` or `None`."
            .to_string()
    } else {
        "Use a stored record field, an active enum payload field, `.value` on Result.Ok, Result.Error, or Option.Some, or a qualified single-payload variant such as `Some.value`."
            .to_string()
    };
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: format!(
            "debug variable target field `{name}` is not a writable payload of `{active}`"
        ),
        hint,
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
