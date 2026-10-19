//! Task-owned live frames and temporary aliases inherited by synchronous callbacks.

use super::TraceHistory;
use fpas_bytecode::{FunctionId, Value};
use std::sync::{Arc, Mutex};

pub(in crate::vm) struct LocalTraceState {
    pub(in crate::vm) commit: u64,
    pub(super) history: Arc<Mutex<TraceHistory>>,
    pub(super) frames: Vec<LocalFrame>,
    pub(super) inherited: Vec<InheritedLocal>,
    pub(super) ended: bool,
}

pub(super) struct LocalFrame {
    pub function: FunctionId,
    pub base: usize,
    pub activation: u64,
    pub live: Vec<LiveLocal>,
}

pub(super) struct LiveLocal {
    pub watch: usize,
    pub binding: usize,
    pub instance: u64,
    pub parameter: Option<ParameterValue>,
}

pub(super) enum ParameterValue {
    /// Immutable admission snapshot of just the selected scalar leaf.
    Frozen(Option<super::model::TraceLeaf>),
    /// A live source cell/reference; it is never a copy of an aggregate root.
    Alias(Value),
}

#[derive(Clone)]
pub(super) struct InheritedLocal {
    pub watch: usize,
    pub instance: u64,
    /// Only a live scalar cell/reference lease, never a copy of an enclosing aggregate.
    pub storage: Value,
}

impl LocalTraceState {
    pub(in crate::vm) fn new(history: Arc<Mutex<TraceHistory>>) -> Self {
        Self {
            commit: 0,
            history,
            frames: Vec::new(),
            inherited: Vec::new(),
            ended: false,
        }
    }

    pub(in crate::vm) fn for_task(&self) -> Self {
        Self::new(Arc::clone(&self.history))
    }

    pub(super) fn retire_from(&mut self, depth: usize) {
        let mut history = self.history.lock().unwrap_or_else(|p| p.into_inner());
        for frame in self.frames.drain(depth..) {
            for live in frame.live {
                history.retire(live.watch, live.instance);
            }
        }
    }
}

impl Drop for LocalTraceState {
    fn drop(&mut self) {
        self.retire_from(0);
    }
}
