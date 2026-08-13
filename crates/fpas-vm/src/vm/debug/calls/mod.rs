//! Detached, effect-checked execution of debugger-side calls.

mod detach;
mod enum_constructor;
mod execute;
mod resolution;

use std::sync::Arc;

use fpas_bytecode::{RuntimeEnumLayout, Value, VerifiedExecutable};

use crate::vm::debug::types::DebugSessionError;

pub(super) use execute::CallSandbox;

/// Construct one detached enum value from verified variant metadata and field values.
pub(in crate::vm::debug) fn construct_enum(
    executable: &VerifiedExecutable,
    layout: Arc<RuntimeEnumLayout>,
    arguments: Vec<Value>,
    max_depth: usize,
    max_detached_values: usize,
) -> Result<Value, DebugSessionError> {
    let mut detacher = detach::ValueDetacher::new(max_detached_values);
    enum_constructor::construct(executable, layout, arguments, &mut detacher, max_depth)
}

/// Construct one detached single-payload enum value from verified variant metadata.
pub(in crate::vm::debug) fn construct_enum_payload(
    executable: &VerifiedExecutable,
    layout: Arc<RuntimeEnumLayout>,
    payload: Value,
    max_depth: usize,
    max_detached_values: usize,
) -> Result<Value, DebugSessionError> {
    construct_enum(
        executable,
        layout,
        vec![payload],
        max_depth,
        max_detached_values,
    )
}
