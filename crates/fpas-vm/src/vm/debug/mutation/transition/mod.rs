//! Qualified variant-suffix resolution and complete wrapper construction.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod diagnostics;
mod suffix;

use std::sync::Arc;

use fpas_bytecode::{DebugTypeId, RuntimeEnumLayout, Value, VerifiedExecutable};

use super::super::evaluation::DebugEvaluationLimits;
use super::super::inspection::MutationPath;
use super::super::types::DebugSessionError;
use super::model::DebugAssignmentSelector;
use super::validate;
use suffix::{resolve_enum_suffix, resolve_option_suffix, resolve_result_suffix};

/// Complete inactive-variant construction after one payload evaluation.
#[derive(Clone)]
pub(in crate::vm::debug) struct TransitionSpec {
    /// Declared type of the single payload slot.
    pub payload_type: DebugTypeId,
    /// Wrapper to construct around the validated payload.
    pub kind: TransitionKind,
}

/// One complete target variant that can be built from a single payload.
#[derive(Clone)]
pub(in crate::vm::debug) enum TransitionKind {
    /// Data-enum variant with exactly one associated field.
    Enum {
        /// Runtime layout used to build the detached enum value.
        layout: Arc<RuntimeEnumLayout>,
    },
    /// `Result.Ok` wrapper.
    ResultOk,
    /// `Result.Error` wrapper.
    ResultError,
    /// `Option.Some` wrapper.
    OptionSome,
}

/// Interpretation of a variant-qualified selector suffix.
pub(in crate::vm::debug) enum QualifiedSuffix {
    /// The named variant is already active; continue as payload replacement.
    ActivePayload {
        /// Existing writable payload path component.
        component: MutationPath,
        /// Declared payload type.
        expected: DebugTypeId,
        /// Number of field selectors consumed, always two.
        consumed: usize,
    },
    /// Replace the wrapper with one newly constructed complete variant.
    Switch(TransitionSpec),
}

/// Exact qualified suffix or the diagnostic used when active-payload lookup also fails.
pub(in crate::vm::debug) enum SuffixResolution {
    /// The first selector names an exact variant and takes precedence over payload field names.
    Exact(QualifiedSuffix),
    /// The first selector is not a variant name on this wrapper.
    Unmatched(DebugSessionError),
}

/// Resolve an exact `Variant.payload` suffix on an enum, `Result`, or `Option`.
///
/// Returns `Ok(None)` when the live value is not a wrapper that this package
/// interprets, so the caller can keep the original active-payload error.
pub(in crate::vm::debug) fn resolve_suffix(
    executable: &fpas_bytecode::Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<Option<SuffixResolution>, DebugSessionError> {
    match current {
        Value::Enum(_) => resolve_enum_suffix(executable, current, expected, remaining).map(Some),
        Value::ResultOk(_) | Value::ResultError(_) => {
            resolve_result_suffix(executable, current, expected, remaining).map(Some)
        }
        Value::OptionSome(_) | Value::OptionNone => {
            resolve_option_suffix(executable, current, expected, remaining).map(Some)
        }
        _ => Ok(None),
    }
}

/// Validate one payload and wrap it as the complete target variant.
pub(in crate::vm::debug) fn construct(
    executable: &VerifiedExecutable,
    spec: TransitionSpec,
    payload: Value,
    limits: DebugEvaluationLimits,
) -> Result<Value, DebugSessionError> {
    validate::value(
        executable.executable(),
        spec.payload_type,
        &payload,
        limits.max_depth,
    )?;
    match spec.kind {
        TransitionKind::ResultOk => Ok(Value::result_ok(payload)),
        TransitionKind::ResultError => Ok(Value::result_error(payload)),
        TransitionKind::OptionSome => Ok(Value::option_some(payload)),
        TransitionKind::Enum { layout } => super::super::construct_enum_payload(
            executable,
            layout,
            payload,
            limits.max_depth,
            limits.max_detached_values,
        ),
    }
}
