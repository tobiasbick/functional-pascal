//! Bounded cycle-safe structural equality of portable debugger types.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::collections::HashSet;

use fpas_bytecode::{DebugType, DebugTypeId};

use super::super::types::{DebugErrorKind, DebugSessionError};

/// Diagnostic nouns for depth, value, and missing-type failures.
#[derive(Debug, Clone, Copy)]
pub(in crate::vm::debug) struct TypeLimitWording {
    /// Noun used in depth and value-limit messages, for example `function signature`.
    pub subject: &'static str,
    /// Phrase used in missing-type diagnostics, for example `function assignment`.
    pub assignment: &'static str,
}

impl TypeLimitWording {
    pub(in crate::vm::debug) const FUNCTION_SIGNATURE: Self = Self {
        subject: "function signature",
        assignment: "function assignment",
    };

    pub(in crate::vm::debug) const TASK_RESULT: Self = Self {
        subject: "task result type",
        assignment: "task assignment",
    };
}

/// Prove two portable types are structurally equal under explicit traversal bounds.
pub(in crate::vm::debug) fn require_equal_bounded(
    types: &[DebugType],
    left: DebugTypeId,
    right: DebugTypeId,
    max_depth: usize,
    max_values: usize,
    wording: TypeLimitWording,
) -> Result<(), DebugSessionError> {
    let mut values = 0_usize;
    let mut visiting = HashSet::new();
    if structurally_equal(
        types,
        left,
        right,
        0,
        max_depth,
        &mut values,
        max_values,
        &mut visiting,
        wording,
    )? {
        Ok(())
    } else {
        Err(DebugSessionError {
            kind: DebugErrorKind::VariableValueType,
            message: format!(
                "debug {} is rejected: source and destination types do not match",
                wording.assignment
            ),
            hint: format!(
                "Assign from a binding whose declared {} matches the destination.",
                wording.subject
            ),
        })
    }
}

/// Look up one portable type id, using assignment-specific missing-type wording.
pub(in crate::vm::debug) fn lookup(
    types: &[DebugType],
    id: DebugTypeId,
    wording: TypeLimitWording,
) -> Result<&DebugType, DebugSessionError> {
    types
        .get(id.get() as usize)
        .ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::VariableValueType,
            message: format!(
                "debug {} refers to unavailable portable type #{}",
                wording.assignment,
                id.get()
            ),
            hint: "Use source and destination bindings whose types are retained in the executable."
                .to_string(),
        })
}

/// Compare two portable types structurally under depth, value, and cycle budgets.
#[expect(
    clippy::too_many_arguments,
    reason = "recursive structural comparison threads depth, value budget, cycle state, and diagnostics"
)]
pub(in crate::vm::debug) fn structurally_equal(
    types: &[DebugType],
    left: DebugTypeId,
    right: DebugTypeId,
    depth: usize,
    max_depth: usize,
    values: &mut usize,
    max_values: usize,
    visiting: &mut HashSet<(u32, u32)>,
    wording: TypeLimitWording,
) -> Result<bool, DebugSessionError> {
    if depth > max_depth {
        return Err(DebugSessionError {
            kind: DebugErrorKind::EvaluationLimit,
            message: format!("debug {} exceeds depth limit {max_depth}", wording.subject),
            hint: format!(
                "Use a shallower {}, or raise the evaluation depth limit.",
                wording.subject
            ),
        });
    }
    *values = values.saturating_add(1);
    if *values > max_values {
        return Err(DebugSessionError {
            kind: DebugErrorKind::EvaluationLimit,
            message: format!(
                "debug {} exceeds detached-value limit {max_values}",
                wording.subject
            ),
            hint: format!(
                "Use a smaller {}, or raise the evaluation value limit.",
                wording.subject
            ),
        });
    }
    if left == right {
        return Ok(true);
    }
    let pair = (left.get(), right.get());
    if !visiting.insert(pair) {
        return Ok(true);
    }
    Ok(
        match (
            lookup(types, left, wording)?,
            lookup(types, right, wording)?,
        ) {
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
                wording,
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
                    wording,
                )? && structurally_equal(
                    types,
                    *left_value,
                    *right_value,
                    depth.saturating_add(1),
                    max_depth,
                    values,
                    max_values,
                    visiting,
                    wording,
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
                    wording,
                )? && structurally_equal(
                    types,
                    *left_error,
                    *right_error,
                    depth.saturating_add(1),
                    max_depth,
                    values,
                    max_values,
                    visiting,
                    wording,
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
                                    wording,
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
                        wording,
                    )?
            }
            (DebugType::Record(left), DebugType::Record(right)) => left == right,
            (DebugType::Enum(left), DebugType::Enum(right)) => left == right,
            _ => false,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_task_result_types_are_equal() {
        let types = vec![
            DebugType::Integer,
            DebugType::Task(DebugTypeId::new(0)),
            DebugType::Task(DebugTypeId::new(0)),
        ];
        require_equal_bounded(
            &types,
            DebugTypeId::new(1),
            DebugTypeId::new(2),
            64,
            65_536,
            TypeLimitWording::TASK_RESULT,
        )
        .expect("matching tasks");
    }

    #[test]
    fn mismatched_task_result_types_are_rejected() {
        let types = vec![
            DebugType::Integer,
            DebugType::Boolean,
            DebugType::Task(DebugTypeId::new(0)),
            DebugType::Task(DebugTypeId::new(1)),
        ];
        let error = require_equal_bounded(
            &types,
            DebugTypeId::new(2),
            DebugTypeId::new(3),
            64,
            65_536,
            TypeLimitWording::TASK_RESULT,
        )
        .expect_err("mismatch");
        assert_eq!(error.kind, DebugErrorKind::VariableValueType);
        assert!(error.message.contains("task assignment"), "{error:?}");
        assert!(!error.hint.contains("<task"), "{}", error.hint);
    }

    #[test]
    fn task_result_comparison_obeys_depth_limits() {
        let types = vec![
            DebugType::Integer,
            DebugType::Task(DebugTypeId::new(0)),
            DebugType::Task(DebugTypeId::new(0)),
        ];
        let error = require_equal_bounded(
            &types,
            DebugTypeId::new(1),
            DebugTypeId::new(2),
            0,
            64,
            TypeLimitWording::TASK_RESULT,
        )
        .expect_err("depth");
        assert_eq!(error.kind, DebugErrorKind::EvaluationLimit);
        assert!(error.message.contains("task result type"), "{error:?}");
    }
}
