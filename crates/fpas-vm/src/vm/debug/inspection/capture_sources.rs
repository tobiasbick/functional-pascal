//! Exact owner-frame reads for debugger-constructed capturing routines.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{DebugBindingId, DebugCaptureSource, FunctionId, Value};

use super::snapshot::{FrameSnapshot, InspectionSnapshot};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl InspectionSnapshot {
    /// Read every capture source from the selected owner frame in ABI order.
    pub(in crate::vm::debug) fn read_value_captures(
        &self,
        frame_id: Option<u64>,
        owner: FunctionId,
        sources: &[DebugCaptureSource],
    ) -> Result<Vec<Value>, DebugSessionError> {
        let frame = self.owner_frame(frame_id, owner)?;
        sources
            .iter()
            .map(|source| {
                let captured = self.capture_binding(frame, source.binding)?;
                if captured.cell_backed || captured.ty != source.ty {
                    return Err(DebugSessionError {
                        kind: DebugErrorKind::VariableValueType,
                        message: "debug capturing routine assignment source binding does not match capture provenance"
                            .to_string(),
                        hint: "Assign a named nested routine whose captures are immutable values of the recorded types."
                            .to_string(),
                    });
                }
                captured.value.clone().ok_or_else(uninitialized_capture)
            })
            .collect()
    }

    fn owner_frame(
        &self,
        frame_id: Option<u64>,
        owner: FunctionId,
    ) -> Result<&FrameSnapshot, DebugSessionError> {
        let Some(frame_id) = frame_id else {
            return Err(DebugSessionError {
                kind: DebugErrorKind::UnknownFrame,
                message: "debug capturing routine assignment requires a selected lexical-owner frame"
                    .to_string(),
                hint: "Select the live frame of the nested routine's enclosing function, then assign the routine name."
                    .to_string(),
            });
        };
        let frame = self
            .frames
            .iter()
            .find(|frame| frame.frame.id == frame_id)
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::UnknownFrame,
                message: format!("debug frame {frame_id} is unknown or expired"),
                hint: "Request stack frames again for the current stop.".to_string(),
            })?;
        if frame.function != owner {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableValueType,
                message: format!(
                    "debug capturing routine assignment requires the selected frame to execute lexical owner {}",
                    owner.get()
                ),
                hint: "Select the live activation of the nested routine's enclosing function. The debugger does not search older, peer-task, or similarly named frames."
                    .to_string(),
            });
        }
        Ok(frame)
    }

    fn capture_binding<'a>(
        &self,
        frame: &'a FrameSnapshot,
        binding: DebugBindingId,
    ) -> Result<&'a super::snapshot::FrameBinding, DebugSessionError> {
        let index = usize::try_from(binding.get()).unwrap_or(usize::MAX);
        let captured = frame.bindings.get(index).ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::VariableValueType,
            message: format!(
                "debug capturing routine assignment source binding {} is missing from the selected frame",
                binding.get()
            ),
            hint: "Assign a routine whose capture metadata matches the current executable. Do not infer a substitute from a display name."
                .to_string(),
        })?;
        if captured.hidden {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableValueType,
                message: "debug capturing routine assignment rejects a hidden owner binding"
                    .to_string(),
                hint:
                    "Assign a nested routine whose captures are ordinary source-visible bindings."
                        .to_string(),
            });
        }
        if !captured.visible {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableValueType,
                message: "debug capturing routine assignment source binding is not visible at the selected stop"
                    .to_string(),
                hint: "Stop where the captured owner binding is still in lexical scope."
                    .to_string(),
            });
        }
        if !captured.initialized {
            return Err(uninitialized_capture());
        }
        Ok(captured)
    }
}

fn uninitialized_capture() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::UninitializedValue,
        message: "debug capturing routine assignment source binding is uninitialized".to_string(),
        hint: "Stop after the captured owner binding has received a value.".to_string(),
    }
}
