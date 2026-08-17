//! Stop-local inspection state shared by focused capture and handle modules.

use std::collections::HashMap;
use std::sync::Arc;

use fpas_bytecode::VerifiedExecutable;

use super::model::{DebugFrame, DebugInspectionLimits, DebugScope};
use super::render::RetainedValue;

pub(crate) struct InspectionSnapshot {
    pub(super) generation: u32,
    pub(super) executable: Arc<VerifiedExecutable>,
    pub(super) frames: Vec<FrameSnapshot>,
    pub(super) globals: Vec<RetainedValue>,
    pub(super) total_frames: usize,
    pub(super) handles: Vec<HandleEntry>,
    pub(super) child_handles: HashMap<(u64, usize), u64>,
    pub(super) limits: DebugInspectionLimits,
}

pub(super) struct FrameSnapshot {
    pub(super) frame: DebugFrame,
    pub(super) function: fpas_bytecode::FunctionId,
    pub(super) scopes: Vec<DebugScope>,
    pub(super) evaluation_values: Vec<RetainedValue>,
    pub(super) bindings: Vec<FrameBinding>,
}

/// One owner-function binding captured from a live frame, indexed by debug binding ID.
pub(super) struct FrameBinding {
    pub(super) initialized: bool,
    pub(super) value: Option<fpas_bytecode::Value>,
    pub(super) ty: fpas_bytecode::DebugTypeId,
    pub(super) kind: fpas_bytecode::DebugBindingKind,
    pub(super) hidden: bool,
    pub(super) cell_backed: bool,
    pub(super) visible: bool,
}

pub(super) struct HandleEntry {
    pub(super) id: u64,
    pub(super) values: Vec<RetainedValue>,
}

pub(super) fn item_id(generation: u32, index: usize) -> u64 {
    (u64::from(generation) << 32) | u64::try_from(index.saturating_add(1)).unwrap_or(u64::MAX)
}

impl InspectionSnapshot {
    /// Return the stop-local generation encoded into frames and variable references.
    pub(in crate::vm::debug) const fn generation(&self) -> u32 {
        self.generation
    }

    /// Return the function identity captured for one current-stop frame.
    pub(in crate::vm::debug) fn frame_function(
        &self,
        frame_id: Option<u64>,
    ) -> Option<fpas_bytecode::FunctionId> {
        let frame_id = frame_id?;
        self.frames
            .iter()
            .find(|frame| frame.frame.id == frame_id)
            .map(|frame| frame.function)
    }
}
