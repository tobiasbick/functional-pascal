//! Portable function parameter modes checked for detached debugger calls.
//! See `docs/pascal/tools/debugger.md`.

use fpas_bytecode::{DebugTypeId, Executable, SharedFunction};

use super::{routine, signature};
use crate::vm::debug::types::DebugSessionError;

/// Checks a runtime function against a declared signature, including bound receivers.
pub(in crate::vm::debug::mutation) fn validate(
    executable: &Executable,
    function: &SharedFunction,
    expected: DebugTypeId,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    let info = executable
        .functions
        .get(function.function.get() as usize)
        .ok_or_else(|| {
            routine::type_error(
                "function value references a missing routine",
                "Rebuild the executable.",
            )
        })?;
    let (mut parameters, result) = routine::portable_signature(executable, info, &function.name)?;
    if function.bound_receiver.is_some() {
        if parameters.is_empty() {
            return Err(routine::type_error(
                "bound function has no receiver parameter",
                "Rebuild the executable.",
            ));
        }
        parameters.remove(0);
    }
    signature::require_signature(
        &executable.debug_types,
        &parameters,
        result,
        expected,
        max_depth,
        65_536,
    )
}
