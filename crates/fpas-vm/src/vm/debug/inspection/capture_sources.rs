//! Exact owner-frame reads for debugger-constructed capturing routines.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{
    DebugBindingId, DebugBindingKind, DebugCaptureKind, DebugCaptureSource, FunctionId, Value,
};

use super::snapshot::{FrameSnapshot, InspectionSnapshot};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl InspectionSnapshot {
    /// Read every capture source from the selected owner frame in ABI order.
    pub(in crate::vm::debug) fn read_captures(
        &self,
        frame_id: Option<u64>,
        owner: FunctionId,
        sources: &[DebugCaptureSource],
    ) -> Result<Vec<Value>, DebugSessionError> {
        let frame = self.owner_frame(frame_id, owner)?;
        sources
            .iter()
            .map(|source| self.read_capture(frame, source))
            .collect()
    }

    fn read_capture(
        &self,
        frame: &FrameSnapshot,
        source: &DebugCaptureSource,
    ) -> Result<Value, DebugSessionError> {
        let captured = self.capture_binding(frame, source.binding)?;
        if captured.ty != source.ty {
            return Err(type_mismatch(source.kind));
        }
        match source.kind {
            DebugCaptureKind::Value => {
                if captured.cell_backed {
                    return Err(type_mismatch(source.kind));
                }
                captured.value.clone().ok_or_else(uninitialized_capture)
            }
            DebugCaptureKind::Cell => {
                require_cell_backed(captured, DebugBindingKind::Local, "direct mutable cell")
            }
            DebugCaptureKind::EnclosingCell => require_cell_backed(
                captured,
                DebugBindingKind::Capture,
                "enclosing mutable cell",
            ),
        }
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

fn require_cell_backed(
    captured: &super::snapshot::FrameBinding,
    expected_kind: DebugBindingKind,
    label: &str,
) -> Result<Value, DebugSessionError> {
    let kind_ok = match expected_kind {
        DebugBindingKind::Capture => captured.kind == DebugBindingKind::Capture,
        _ => captured.kind != DebugBindingKind::Capture,
    };
    if !captured.cell_backed || !kind_ok {
        return Err(DebugSessionError {
            kind: DebugErrorKind::VariableValueType,
            message: format!(
                "debug capturing routine assignment source binding is not a {label} capture"
            ),
            hint:
                "Assign a named nested routine whose mutable captures are the original owner cells."
                    .to_string(),
        });
    }
    match &captured.value {
        Some(Value::Cell(cell)) => Ok(Value::Cell(std::sync::Arc::clone(cell))),
        _ => Err(DebugSessionError {
            kind: DebugErrorKind::VariableValueType,
            message: format!(
                "debug capturing routine assignment source binding does not hold a {label} handle"
            ),
            hint: "Assign a named nested routine whose mutable captures are live cell handles, not copied payloads."
                .to_string(),
        }),
    }
}

fn type_mismatch(kind: DebugCaptureKind) -> DebugSessionError {
    let hint = match kind {
        DebugCaptureKind::Value => {
            "Assign a named nested routine whose captures are immutable values of the recorded types."
        }
        DebugCaptureKind::Cell | DebugCaptureKind::EnclosingCell => {
            "Assign a named nested routine whose mutable captures match the recorded cell identities."
        }
    };
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message:
            "debug capturing routine assignment source binding does not match capture provenance"
                .to_string(),
        hint: hint.to_string(),
    }
}

fn uninitialized_capture() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::UninitializedValue,
        message: "debug capturing routine assignment source binding is uninitialized".to_string(),
        hint: "Stop after the captured owner binding has received a value.".to_string(),
    }
}
