//! Stop-local inspection state shared by focused capture and handle modules.

use std::collections::HashMap;
use std::sync::Arc;

use crate::{MemoryDomain, VerifiedExecutable};

use super::model::{DebugFrame, DebugInspectionLimits, DebugScope};
use super::render::RetainedValue;

pub(crate) struct InspectionSnapshot {
    pub(super) generation: u32,
    pub(in crate::execution::vm::debug) domain: MemoryDomain,
    pub(super) executable: Arc<VerifiedExecutable>,
    pub(super) frames: Vec<FrameSnapshot>,
    pub(super) globals: Vec<RetainedValue>,
    pub(super) total_frames: usize,
    pub(super) handles: Vec<HandleEntry>,
    pub(super) child_handles: HashMap<(u64, usize), u64>,
    pub(super) limits: DebugInspectionLimits,
    pub(super) roots: Vec<crate::RootValues>,
}

pub(super) struct FrameSnapshot {
    pub(super) frame: DebugFrame,
    pub(super) function: crate::FunctionId,
    pub(super) scopes: Vec<DebugScope>,
    pub(super) evaluation_values: Vec<RetainedValue>,
    pub(super) bindings: Vec<FrameBinding>,
}

/// One owner-function binding captured from a live frame, indexed by debug binding ID.
pub(super) struct FrameBinding {
    pub(super) initialized: bool,
    pub(super) value: Option<crate::Value>,
    pub(super) ty: crate::DebugTypeId,
    pub(super) kind: crate::DebugBindingKind,
    pub(super) hidden: bool,
    pub(super) cell_backed: bool,
    pub(super) visible: bool,
}

pub(super) struct HandleEntry {
    pub(super) id: u64,
    pub(super) values: Vec<RetainedValue>,
    pub(super) _roots: Vec<crate::RootValues>,
}

pub(super) fn item_id(generation: u32, index: usize) -> u64 {
    (u64::from(generation) << 32) | u64::try_from(index.saturating_add(1)).unwrap_or(u64::MAX)
}

impl InspectionSnapshot {
    /// Return the stop-local generation encoded into frames and variable references.
    pub(in crate::execution::vm::debug) const fn generation(&self) -> u32 {
        self.generation
    }

    /// Return the function identity captured for one current-stop frame.
    pub(in crate::execution::vm::debug) fn frame_function(
        &self,
        frame_id: Option<u64>,
    ) -> Option<crate::FunctionId> {
        let frame_id = frame_id?;
        self.frames
            .iter()
            .find(|frame| frame.frame.id == frame_id)
            .map(|frame| frame.function)
    }
}
