//! Cycle-safe structural compatibility of portable debugger function types.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::collections::HashSet;

use fpas_bytecode::{DebugType, DebugTypeId};

use super::super::super::types::{DebugErrorKind, DebugSessionError};

/// Prove two portable types are structurally compatible as function signatures.
pub(super) fn require_compatible(
    types: &[DebugType],
    source: DebugTypeId,
    destination: DebugTypeId,
) -> Result<(), DebugSessionError> {
    let source_ty = lookup(types, source)?;
    let destination_ty = lookup(types, destination)?;
    match (source_ty, destination_ty) {
        (
            DebugType::Function {
                parameters: source_parameters,
                result: source_result,
            },
            DebugType::Function {
                parameters: destination_parameters,
                result: destination_result,
            },
        ) => {
            if source_parameters.len() != destination_parameters.len() {
                return Err(signature_mismatch(
                    "parameter count does not match the destination function type",
                ));
            }
            let mut visiting = HashSet::new();
            for (index, (left, right)) in source_parameters
                .iter()
                .zip(destination_parameters)
                .enumerate()
            {
                if !structurally_equal(types, *left, *right, &mut visiting)? {
                    return Err(signature_mismatch(&format!(
                        "parameter {} does not match the destination function type",
                        index.saturating_add(1)
                    )));
                }
            }
            if structurally_equal(types, *source_result, *destination_result, &mut visiting)? {
                Ok(())
            } else {
                Err(signature_mismatch(
                    "result type does not match the destination function type",
                ))
            }
        }
        (DebugType::Function { .. }, _) => Err(signature_mismatch(
            "destination does not have portable function type metadata",
        )),
        (_, DebugType::Function { .. }) => Err(signature_mismatch(
            "source does not have portable function type metadata",
        )),
        _ => Err(signature_mismatch(
            "source and destination are not declared function types",
        )),
    }
}

fn structurally_equal(
    types: &[DebugType],
    left: DebugTypeId,
    right: DebugTypeId,
    visiting: &mut HashSet<(u32, u32)>,
) -> Result<bool, DebugSessionError> {
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
        | (DebugType::Task(left), DebugType::Task(right)) => {
            structurally_equal(types, *left, *right, visiting)?
        }
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
            structurally_equal(types, *left_key, *right_key, visiting)?
                && structurally_equal(types, *left_value, *right_value, visiting)?
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
            structurally_equal(types, *left_ok, *right_ok, visiting)?
                && structurally_equal(types, *left_error, *right_error, visiting)?
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
                        Ok(matches && structurally_equal(types, *left, *right, visiting)?)
                    },
                )?
                && structurally_equal(types, *left_result, *right_result, visiting)?
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
        hint: "Use a source binding whose function signature matches the destination parameter order and result type."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpas_bytecode::{EnumTypeId, RecordTypeId};

    fn types(entries: Vec<DebugType>) -> Vec<DebugType> {
        entries
    }

    #[test]
    fn matching_function_types_are_compatible() {
        let types = types(vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
        ]);
        require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(1)).expect("same node");
    }

    #[test]
    fn parameter_count_mismatch_is_rejected() {
        let types = types(vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
            DebugType::Function {
                parameters: Vec::new(),
                result: DebugTypeId::new(0),
            },
        ]);
        let error = require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(2))
            .expect_err("arity");
        assert_eq!(error.kind, DebugErrorKind::VariableValueType);
        assert!(error.message.contains("parameter count"), "{error:?}");
        assert!(error.hint.contains("signature"), "{}", error.hint);
    }

    #[test]
    fn parameter_and_result_mismatches_are_rejected() {
        let types = types(vec![
            DebugType::Integer,
            DebugType::Boolean,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
            DebugType::Function {
                parameters: vec![DebugTypeId::new(1)],
                result: DebugTypeId::new(0),
            },
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(1),
            },
        ]);
        let parameter = require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(3))
            .expect_err("param");
        assert!(parameter.message.contains("parameter 1"), "{parameter:?}");
        let result = require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(4))
            .expect_err("result");
        assert!(result.message.contains("result type"), "{result:?}");
    }

    #[test]
    fn nested_function_and_layout_identity_are_compared_structurally() {
        let types = types(vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
            DebugType::Function {
                parameters: vec![DebugTypeId::new(1)],
                result: DebugTypeId::new(0),
            },
            DebugType::Function {
                parameters: vec![DebugTypeId::new(5)],
                result: DebugTypeId::new(0),
            },
            DebugType::Record(RecordTypeId::new(0)),
            DebugType::Function {
                parameters: vec![DebugTypeId::new(4)],
                result: DebugTypeId::new(0),
            },
            DebugType::Record(RecordTypeId::new(1)),
            DebugType::Enum(EnumTypeId::new(0)),
            DebugType::Function {
                parameters: vec![DebugTypeId::new(7)],
                result: DebugTypeId::new(0),
            },
            DebugType::Enum(EnumTypeId::new(1)),
            DebugType::Function {
                parameters: vec![DebugTypeId::new(9)],
                result: DebugTypeId::new(0),
            },
        ]);
        require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(2)).expect("nested same");
        assert!(
            require_compatible(&types, DebugTypeId::new(2), DebugTypeId::new(3))
                .expect_err("nested")
                .message
                .contains("parameter 1")
        );
        assert!(
            require_compatible(&types, DebugTypeId::new(5), DebugTypeId::new(3))
                .expect_err("record")
                .hint
                .contains("signature")
        );
        assert!(
            require_compatible(&types, DebugTypeId::new(8), DebugTypeId::new(10))
                .expect_err("enum")
                .kind
                == DebugErrorKind::VariableValueType
        );
    }

    #[test]
    fn recursive_function_graphs_terminate() {
        let types = types(vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(2),
            },
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(1),
            },
        ]);
        require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(2)).expect("recursive");
    }

    #[test]
    fn malformed_type_ids_are_rejected() {
        let types = types(vec![DebugType::Integer]);
        let error = require_compatible(&types, DebugTypeId::new(1), DebugTypeId::new(1))
            .expect_err("missing");
        assert_eq!(error.kind, DebugErrorKind::VariableValueType);
        assert!(error.message.contains("unavailable"), "{error:?}");
    }
}
