//! Detached data delivered to an opted-in VM host.

use super::{FailureDiagnostic, FailureValue, Lease};
use crate::{DebugBindingKind, DebugSourceLocation};
use std::sync::Arc;

/// One source-visible binding captured before task cleanup.
#[derive(Debug)]
pub struct FailureBinding {
    /// Source spelling of the binding.
    pub name: String,
    /// Parameter, local, or lexical capture.
    pub kind: DebugBindingKind,
    /// Function-local lexical scope identity, preserving shadowing.
    pub scope_id: u32,
    /// Detached immutable value, including explicit unavailable/truncated markers.
    pub value: FailureValue,
}

/// One available FPAS frame at the original fault or hosted continuation.
#[derive(Debug)]
pub struct FailureFrame {
    /// Zero-based depth; the failing frame is first.
    pub depth: usize,
    /// Stored routine name, without a generated entry prefix.
    pub function: String,
    /// Portable image source position; no host path resolution is performed.
    pub location: Option<DebugSourceLocation>,
    /// Source bindings in metadata declaration order.
    pub bindings: Vec<FailureBinding>,
    /// Stable reason for omitted frame data, when any.
    pub reason: Option<&'static str>,
}

/// Immutable original-cause data whose storage follows its owning diagnostic.
pub struct FailureSnapshot {
    /// VM-local failure identity, preserved through clones; zero for a later peer observation.
    pub id: u64,
    /// Responsible task, or unavailable for diagnostics raised outside execution.
    pub task_id: Option<u64>,
    /// Original task entry, retained even after Wait releases its scheduler record.
    pub(super) entry_function: Option<crate::FunctionId>,
    /// Available frames captured before cleanup, never a waiter's substituted stack.
    pub frames: Vec<FailureFrame>,
    /// Total known frame count, including frames omitted by a bound.
    pub total_frames: Option<usize>,
    /// Stable reason for incomplete or unavailable stack data.
    pub reason: Option<&'static str>,
    /// Shared value/model storage reservation released with the detached snapshot.
    pub(super) lease: Lease,
}

impl std::fmt::Debug for FailureSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FailureSnapshot")
            .field("id", &self.id)
            .field("task_id", &self.task_id)
            .field("frames", &self.frames)
            .field("total_frames", &self.total_frames)
            .field("reason", &self.reason)
            .finish()
    }
}

/// Terminal observation of the VM's selected error, before remaining shutdown joins.
/// Receiving this data does not change ownership barriers or cancel/stop the VM.
#[derive(Debug)]
pub struct FailureObservation {
    /// Present on the selected top-level observation; secondary errors share this catalog.
    pub task_catalog: Option<Arc<super::FailureTaskCatalog>>,
    /// The selected diagnostic; any omitted text has an explicit reason.
    pub error: Arc<FailureDiagnostic>,
    /// The original responsible task's state.
    pub cause: Arc<FailureSnapshot>,
    /// Structured secondary failures in the runtime's existing attachment order.
    pub related_errors: Vec<FailureObservation>,
    /// Number of direct secondary errors omitted by model, depth, or relationship bounds.
    pub related_errors_omitted: usize,
    /// Charged storage for this owned observation.
    pub(super) _lease: Lease,
    /// Charged traversal and output collection storage.
    pub(super) _collection_lease: Lease,
}
