//! Bounded immutable task and global observations shared by embedded hosts.

use super::{FailureSnapshot, FailureValue, Lease};
use std::sync::Arc;

/// One current task, never an archived released outcome.
#[derive(Debug)]
pub struct FailureTask {
    /// Runtime task identity; zero is the main task.
    pub id: u64,
    /// Stored entry routine name, when available.
    pub name: Option<String>,
    /// Runnable, running, waiting, sleeping, completed, failed, or cancelled.
    pub state: &'static str,
    /// Original fault or the later cooperative failure-handling observation.
    pub observation: &'static str,
    /// Available, partial, unavailable, or not_applicable.
    pub inspection: &'static str,
    /// Explicit reason for missing frame data.
    pub reason: Option<&'static str>,
    /// Detached frames; completed peers have no fabricated stack.
    pub snapshot: Option<Arc<FailureSnapshot>>,
    /// Charged task-name storage.
    pub(super) _lease: Lease,
}

/// One captured program or unit global.
#[derive(Debug)]
pub struct FailureGlobal {
    /// Stored source binding name.
    pub name: String,
    /// Detached current value, or an explicit unavailable/uninitialized marker.
    pub value: FailureValue,
}

/// Globals are captured once from the actual root worker's storage.
#[derive(Debug)]
pub struct FailureGlobals {
    /// Globals in image declaration order.
    pub bindings: Vec<FailureGlobal>,
    /// Known number, including omitted globals.
    pub total: usize,
    /// Explicit capture omission or unavailability.
    pub reason: Option<&'static str>,
    /// Charged binding collection and value storage.
    pub(super) _lease: Lease,
}

/// A cooperative catalog cut, distinct from the original cause's fault boundary.
#[derive(Debug)]
pub struct FailureTaskCatalog {
    /// Captured tasks in ascending runtime identity order.
    pub tasks: Vec<FailureTask>,
    /// All live and retained terminal identities at the cut, including omitted tasks.
    pub total_tasks: usize,
    /// Number omitted by task or storage bounds.
    pub omitted_tasks: usize,
    /// Shared global capture; unavailable storage is represented explicitly.
    pub globals: Option<Arc<FailureGlobals>>,
    /// Available, partial, or unavailable global data.
    pub globals_status: &'static str,
    /// Explicit collection/model omission reason.
    pub reason: Option<&'static str>,
    /// Charged collection metadata and temporary selection storage.
    pub(super) _lease: Lease,
}

/// Find explicit missing or truncated data within a detached value tree.
pub(super) fn value_reason(value: &FailureValue) -> Option<&'static str> {
    value.reason.or_else(|| {
        value
            .children
            .iter()
            .find_map(|(_, value)| value_reason(value))
    })
}

/// Find explicit missing or truncated frame or binding data.
pub(super) fn snapshot_reason(snapshot: &FailureSnapshot) -> Option<&'static str> {
    snapshot.reason.or_else(|| {
        snapshot.frames.iter().find_map(|frame| {
            frame.reason.or_else(|| {
                frame
                    .bindings
                    .iter()
                    .find_map(|binding| value_reason(&binding.value))
            })
        })
    })
}
