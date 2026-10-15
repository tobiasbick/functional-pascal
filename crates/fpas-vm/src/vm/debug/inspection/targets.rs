//! Stop-local writable origins retained alongside immutable presentation values.

use std::sync::{Arc, Mutex};

use fpas_bytecode::{DebugTypeId, Value};

use crate::vm::debug::initializer_suppression::SourceInitializerTarget;

pub(in crate::vm::debug) use crate::vm::inspection::values::{
    ValuePath as MutationPath,
    payload::{PayloadError, active_label, resolve as resolve_payload},
};

/// Stop-local writability retained separately from value presentation.
#[derive(Clone)]
pub(in crate::vm::debug) enum MutationAccess {
    Writable(MutationTarget),
    NotMutable,
    Unsupported,
    Unavailable,
}

/// Typed storage path owned by one debugger generation.
#[derive(Clone)]
pub(in crate::vm::debug) struct MutationTarget {
    pub root: MutationRoot,
    pub path: Vec<MutationPath>,
    pub expected_type: DebugTypeId,
    pub generation: u32,
    pub frame_id: Option<u64>,
    /// Whether live storage already holds a value at capture time.
    pub initialized: bool,
    pub initializer: Option<SourceInitializerTarget>,
}

/// Live storage root used only by debugger mutation adapters.
#[derive(Clone)]
pub(in crate::vm::debug) enum MutationRoot {
    FrameRegister(usize),
    Global(usize),
    ClosureCell(Arc<Mutex<Value>>),
}

impl MutationAccess {
    /// Carry writable storage through one typed structural descendant.
    pub(super) fn descendant(&self, component: MutationPath, expected_type: DebugTypeId) -> Self {
        match self {
            Self::Writable(target) => {
                let mut target = target.clone();
                target.path.push(component);
                target.expected_type = expected_type;
                Self::Writable(target)
            }
            other => other.clone(),
        }
    }
}

impl crate::vm::inspection::values::ValueOrigin for MutationAccess {
    fn unsupported() -> Self {
        Self::Unsupported
    }
    fn descendant(&self, component: MutationPath, expected_type: DebugTypeId) -> Self {
        self.descendant(component, expected_type)
    }
}
