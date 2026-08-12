//! Stop-local writable origins retained alongside immutable presentation values.

mod payload;

use std::sync::{Arc, Mutex};

use fpas_bytecode::{DebugTypeId, EnumVariantId, Value};

pub(in crate::vm::debug) use payload::{PayloadError, active_label, resolve as resolve_payload};

#[derive(Clone)]
pub(in crate::vm::debug) enum MutationAccess {
    Writable(MutationTarget),
    NotMutable,
    Unsupported,
    Unavailable,
}

#[derive(Clone)]
pub(in crate::vm::debug) struct MutationTarget {
    pub root: MutationRoot,
    pub path: Vec<MutationPath>,
    pub expected_type: DebugTypeId,
    pub generation: u32,
    pub frame_id: Option<u64>,
}

#[derive(Clone)]
pub(in crate::vm::debug) enum MutationRoot {
    FrameRegister(usize),
    Global(usize),
    ClosureCell(Arc<Mutex<Value>>),
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::vm::debug) enum MutationPath {
    RecordField(usize),
    ArrayIndex(usize),
    DictionaryValue(Value),
    EnumField {
        variant: EnumVariantId,
        index: usize,
    },
    ResultOk,
    ResultError,
    OptionSome,
}

impl MutationAccess {
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
