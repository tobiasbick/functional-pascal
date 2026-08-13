//! Exact lexical lookup of a visible binding's portable type.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::DebugTypeId;

use super::render::RetainedValue;
use super::snapshot::InspectionSnapshot;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl InspectionSnapshot {
    /// Resolve one visible initialized binding to its declared portable type.
    ///
    /// Frame selection follows ordinary lexical shadowing; omitting a frame searches globals
    /// only. Hidden bindings are already excluded from the captured evaluation environment.
    pub(in crate::vm::debug) fn resolve_binding_type(
        &self,
        frame_id: Option<u64>,
        name: &str,
    ) -> Result<DebugTypeId, DebugSessionError> {
        self.validate_evaluation_frame(frame_id)?;
        let frame_values = match frame_id {
            Some(frame_id) => Some(
                self.frames
                    .iter()
                    .find(|frame| frame.frame.id == frame_id)
                    .map(|frame| frame.evaluation_values.as_slice())
                    .ok_or_else(|| DebugSessionError {
                        kind: DebugErrorKind::UnknownFrame,
                        message: format!("debug frame {frame_id} is unknown or expired"),
                        hint: "Request stack frames again for the current stop.".to_string(),
                    })?,
            ),
            None => None,
        };
        let retained = visible_binding(frame_values, &self.globals, name).ok_or_else(|| {
            DebugSessionError {
                kind: DebugErrorKind::UnknownName,
                message: format!("debug expression name `{name}` is not visible"),
                hint: "Use a parameter, local, capture, or global visible at the selected frame."
                    .to_string(),
            }
        })?;
        retained.value.as_ref().ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::UninitializedValue,
            message: format!("debug expression name `{name}` is uninitialized"),
            hint: "Stop after the binding has received a value.".to_string(),
        })?;
        let ty = retained.debug_type.ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::VariableValueType,
            message: format!(
                "debug source binding `{name}` does not retain portable function type metadata"
            ),
            hint: "Assign from a source-declared function binding, not an evaluation-only result."
                .to_string(),
        })?;
        Ok(ty)
    }
}

/// First visible binding with a case-insensitive name, preferring frame values over globals.
pub(super) fn visible_binding<'a>(
    frame_values: Option<&'a [RetainedValue]>,
    globals: &'a [RetainedValue],
    name: &str,
) -> Option<&'a RetainedValue> {
    frame_values
        .into_iter()
        .flatten()
        .chain(globals)
        .find(|value| value.name.eq_ignore_ascii_case(name))
}
