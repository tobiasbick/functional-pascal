//! Stable diagnostics for seeded empty-storage initialization.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::super::super::types::{DebugErrorKind, DebugSessionError};

pub(in crate::vm::debug) fn root_only(root: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: format!(
            "debug storage initialization target `{root}` has no descendant selector"
        ),
        hint: "Use expression.set or setExpression to assign a complete root value. storage.initialize requires a field, index, or payload descendant."
            .to_string(),
    }
}

pub(in crate::vm::debug) fn already_initialized(root: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::StorageAlreadyInitialized,
        message: format!("debug storage initialization target `{root}` is already initialized"),
        hint: "Use expression.set or setExpression to mutate an already initialized binding."
            .to_string(),
    }
}

pub(in crate::vm::debug) fn unsupported_capture(root: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: format!(
            "debug storage initialization target `{root}` is not an empty mutable local or global"
        ),
        hint: "Select a source-declared mutable local or global whose live storage is still empty."
            .to_string(),
    }
}

pub(in crate::vm::debug) fn identity_bearing(detail: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug storage initializer is rejected: {detail}"),
        hint: "Supply a complete portable seed without functions, tasks, capture cells, or opaque handles."
            .to_string(),
    }
}

pub(in crate::vm::debug) fn unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug variable live storage is unavailable".to_string(),
        hint: "Retry at a stable stop after the live storage becomes available.".to_string(),
    }
}
