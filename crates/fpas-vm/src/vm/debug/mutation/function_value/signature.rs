//! Cycle-safe structural compatibility of portable debugger function types.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::collections::HashSet;

use fpas_bytecode::{DebugType, DebugTypeId};

use super::super::super::types::{DebugErrorKind, DebugSessionError};

/// Prove two portable types are structurally compatible as function signatures.
#[cfg(test)]
pub(super) fn require_compatible(
    types: &[DebugType],
    source: DebugTypeId,
    destination: DebugTypeId,
) -> Result<(), DebugSessionError> {
    require_compatible_bounded(types, source, destination, 64, 65_536)
}

/// Prove two portable function types under explicit traversal bounds.
pub(super) fn require_compatible_bounded(
    types: &[DebugType],
    source: DebugTypeId,
    destination: DebugTypeId,
    max_depth: usize,
    max_values: usize,
) -> Result<(), DebugSessionError> {
    let source_ty = lookup(types, source)?;
    match source_ty {
        DebugType::Function { parameters, result } => require_signature(
            types,
            parameters,
            *result,
            destination,
            max_depth,
            max_values,
        ),
        _ => Err(signature_mismatch(
            "source does not have portable function type metadata",
        )),
    }
}

/// Prove reconstructed parameter and result types match a destination function type.
pub(super) fn require_signature(
    types: &[DebugType],
    source_parameters: &[DebugTypeId],
    source_result: DebugTypeId,
    destination: DebugTypeId,
    max_depth: usize,
    max_values: usize,
) -> Result<(), DebugSessionError> {
    match lookup(types, destination)? {
        DebugType::Function {
            parameters: destination_parameters,
            result: destination_result,
        } => {
            if source_parameters.len() != destination_parameters.len() {
                return Err(signature_mismatch(
                    "parameter count does not match the destination function type",
                ));
            }
            let mut visiting = HashSet::new();
            let mut values = 0_usize;
            for (index, (left, right)) in source_parameters
                .iter()
                .zip(destination_parameters)
                .enumerate()
            {
                if !structurally_equal(
                    types,
                    *left,
                    *right,
                    0,
                    max_depth,
                    &mut values,
                    max_values,
                    &mut visiting,
                )? {
                    return Err(signature_mismatch(&format!(
                        "parameter {} does not match the destination function type",
                        index.saturating_add(1)
                    )));
                }
            }
            if structurally_equal(
                types,
                source_result,
                *destination_result,
                0,
                max_depth,
                &mut values,
                max_values,
                &mut visiting,
            )? {
                Ok(())
            } else {
                Err(signature_mismatch(
                    "result type does not match the destination function type",
                ))
            }
        }
        _ => Err(signature_mismatch(
            "destination does not have portable function type metadata",
        )),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "recursive structural comparison threads depth, value budget, and cycle state"
)]
fn structurally_equal(
    types: &[DebugType],
    left: DebugTypeId,
    right: DebugTypeId,
    depth: usize,
    max_depth: usize,
    values: &mut usize,
    max_values: usize,
    visiting: &mut HashSet<(u32, u32)>,
) -> Result<bool, DebugSessionError> {
    if depth > max_depth {
        return Err(DebugSessionError {
            kind: DebugErrorKind::EvaluationLimit,
            message: format!("debug function signature exceeds depth limit {max_depth}"),
            hint: "Use a shallower function signature, or raise the evaluation depth limit."
                .to_string(),
        });
    }
    *values = values.saturating_add(1);
    if *values > max_values {
        return Err(DebugSessionError {
            kind: DebugErrorKind::EvaluationLimit,
            message: format!("debug function signature exceeds detached-value limit {max_values}"),
            hint: "Use a smaller function signature, or raise the evaluation value limit."
                .to_string(),
        });
    }
    if left == right {
        return Ok(true);
    }
    let pair = (left.get(), right.get());
    if !visiting.insert(pair) {
        return Ok(true);
    }
    Ok(match (lookup(types, left)?, lookup(types, right)?) {
        (DebugType::Unit, DebugType::Unit)
        | (DebugType::Boolean, DebugType::Boolean)
        | (DebugType::Integer, DebugType::Integer)
        | (DebugType::Real, DebugType::Real)
        | (DebugType::String, DebugType::String)
        | (DebugType::Dynamic, DebugType::Dynamic) => true,
        (DebugType::Array(left), DebugType::Array(right))
        | (DebugType::Option(left), DebugType::Option(right))
        | (DebugType::Cell(left), DebugType::Cell(right))
        | (DebugType::Task(left), DebugType::Task(right)) => structurally_equal(
            types,
            *left,
            *right,
            depth.saturating_add(1),
            max_depth,
            values,
            max_values,
            visiting,
        )?,
        (
            DebugType::Dictionary {
                key: left_key,
                value: left_value,
            },
            DebugType::Dictionary {
                key: right_key,
                value: right_value,
            },
        ) => {
            structurally_equal(
                types,
                *left_key,
                *right_key,
                depth.saturating_add(1),
                max_depth,
                values,
                max_values,
                visiting,
            )? && structurally_equal(
                types,
                *left_value,
                *right_value,
                depth.saturating_add(1),
                max_depth,
                values,
                max_values,
                visiting,
            )?
        }
        (
            DebugType::Result {
                ok: left_ok,
                error: left_error,
            },
            DebugType::Result {
                ok: right_ok,
                error: right_error,
            },
        ) => {
            structurally_equal(
                types,
                *left_ok,
                *right_ok,
                depth.saturating_add(1),
                max_depth,
                values,
                max_values,
                visiting,
            )? && structurally_equal(
                types,
                *left_error,
                *right_error,
                depth.saturating_add(1),
                max_depth,
                values,
                max_values,
                visiting,
            )?
        }
        (
            DebugType::Function {
                parameters: left_parameters,
                result: left_result,
            },
            DebugType::Function {
                parameters: right_parameters,
                result: right_result,
            },
        ) => {
            left_parameters.len() == right_parameters.len()
                && left_parameters.iter().zip(right_parameters).try_fold(
                    true,
                    |matches, (left, right)| {
                        Ok(matches
                            && structurally_equal(
                                types,
                                *left,
                                *right,
                                depth.saturating_add(1),
                                max_depth,
                                values,
                                max_values,
                                visiting,
                            )?)
                    },
                )?
                && structurally_equal(
                    types,
                    *left_result,
                    *right_result,
                    depth.saturating_add(1),
                    max_depth,
                    values,
                    max_values,
                    visiting,
                )?
        }
        (DebugType::Record(left), DebugType::Record(right)) => left == right,
        (DebugType::Enum(left), DebugType::Enum(right)) => left == right,
        _ => false,
    })
}

fn lookup(types: &[DebugType], id: DebugTypeId) -> Result<&DebugType, DebugSessionError> {
    types
        .get(id.get() as usize)
        .ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::VariableValueType,
            message: format!(
                "debug function assignment refers to unavailable portable type #{}",
                id.get()
            ),
            hint: "Use source and destination bindings whose types are retained in the executable."
                .to_string(),
        })
}

fn signature_mismatch(detail: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug function assignment is rejected: {detail}"),
        hint: "Use a source binding or non-capturing routine whose function signature matches the destination parameter order and result type."
            .to_string(),
    }
}

#[cfg(test)]
#[path = "signature/tests.rs"]
mod tests;
