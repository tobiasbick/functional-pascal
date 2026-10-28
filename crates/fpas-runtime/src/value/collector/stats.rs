//! Per-domain allocation and explicit reclamation observations, never graph roots.
use std::sync::atomic::{AtomicUsize, Ordering};

/// Scratch bounds for a whole-domain pass; exceeding either leaves every payload intact.
#[derive(Debug, Clone, Copy)]
pub struct CollectionLimits {
    /// Maximum candidate cells retained during preparation.
    pub max_cells: usize,
    /// Maximum values visited during iterative root marking.
    pub max_graph_nodes: usize,
}
impl Default for CollectionLimits {
    fn default() -> Self {
        Self {
            max_cells: usize::MAX,
            max_graph_nodes: usize::MAX,
        }
    }
}
/// Monotonic domain-local counters. A cleared payload need not release an external
/// administrative cell handle; released cells count actual storage destruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionStats {
    /// Successfully allocated cell storage objects.
    pub allocated_cells: usize,
    /// Cell storage objects destroyed while their domain remained alive.
    pub released_cells: usize,
    /// Unreachable cell payloads disconnected by completed passes.
    pub cleared_cells: usize,
    /// Complete successful whole-domain passes, including empty passes.
    pub completed_passes: usize,
    /// Failed preparations or interrupted committed teardown.
    pub aborted_passes: usize,
    /// Current live weak cell registrations, including administrative handles.
    pub live_cells: usize,
}
#[derive(Default)]
pub(in crate::value) struct Counters {
    pub(in crate::value) allocated: AtomicUsize,
    pub(in crate::value) released: AtomicUsize,
    pub(in crate::value) cleared: AtomicUsize,
    pub(in crate::value) completed: AtomicUsize,
    pub(in crate::value) aborted: AtomicUsize,
}
pub(in crate::value) fn increment(counter: &AtomicUsize, amount: usize) {
    let _ = counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
        Some(old.saturating_add(amount))
    });
}
impl super::super::MemoryDomain {
    /// Snapshot allocation/reclamation diagnostics without retaining any value graph.
    #[must_use]
    pub fn collection_stats(&self) -> CollectionStats {
        let counters = &self.0.collection;
        CollectionStats {
            allocated_cells: counters.allocated.load(Ordering::Relaxed),
            released_cells: counters.released.load(Ordering::Relaxed),
            cleared_cells: counters.cleared.load(Ordering::Relaxed),
            completed_passes: counters.completed.load(Ordering::Relaxed),
            aborted_passes: counters.aborted.load(Ordering::Relaxed),
            live_cells: self.registered_count(),
        }
    }
}
