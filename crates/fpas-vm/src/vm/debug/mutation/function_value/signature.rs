//! Cycle-safe structural compatibility of portable debugger function types.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::collections::HashSet;

use fpas_bytecode::{DebugType, DebugTypeId};

use super::super::super::types::{DebugErrorKind, DebugSessionError};
use super::super::portable_type::{self, TypeLimitWording};

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
                if !portable_type::structurally_equal(
                    types,
                    *left,
                    *right,
                    0,
                    max_depth,
                    &mut values,
                    max_values,
                    &mut visiting,
                    TypeLimitWording::FUNCTION_SIGNATURE,
                )? {
                    return Err(signature_mismatch(&format!(
                        "parameter {} does not match the destination function type",
                        index.saturating_add(1)
                    )));
                }
            }
            if portable_type::structurally_equal(
                types,
                source_result,
                *destination_result,
                0,
                max_depth,
                &mut values,
                max_values,
                &mut visiting,
                TypeLimitWording::FUNCTION_SIGNATURE,
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

fn lookup(types: &[DebugType], id: DebugTypeId) -> Result<&DebugType, DebugSessionError> {
    portable_type::lookup(types, id, TypeLimitWording::FUNCTION_SIGNATURE)
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
