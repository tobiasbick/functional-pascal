//! Whole-domain mark and cell-edge removal under the AP48.5 exclusive permit.
//!
//! Only registered roots mark cells. Candidate handles, payload locks and release
//! queues are administrative owners. No reference-count heuristic grants liveness.
mod commit;
mod stats;
#[cfg(test)]
mod tests;

use super::{MemoryDomain, OwnedCell, OwnershipError, Value};
pub use stats::{CollectionLimits, CollectionStats};
pub(in crate::value) use stats::{Counters, increment};

impl MemoryDomain {
    /// Disconnect unreachable cells during a trusted maintenance hook.
    ///
    /// `roots` must be the complete prepared view supplied to `DomainMaintenance`.
    /// This low-level runtime entry never acquires execution or starts maintenance;
    /// callers must already own exclusive collection. Use the maintenance request
    /// route, not this operation, to enter collection. No FPAS callback is invoked.
    /// # Errors
    /// Rejects nonexclusive/poisoned access, incomplete marking, unavailable cells,
    /// foreign roots or exhausted scratch bounds before changing any payload.
    pub fn reclaim_unreachable(
        &self,
        roots: &[Value],
        limits: CollectionLimits,
    ) -> Result<usize, OwnershipError> {
        self.require_collection()?;
        let result = self.prepare_and_clear(roots, limits);
        if result.is_err() {
            increment(&self.0.collection.aborted, 1);
        }
        result
    }
    fn prepare_and_clear(
        &self,
        roots: &[Value],
        limits: CollectionLimits,
    ) -> Result<usize, OwnershipError> {
        let candidates = self.collection_candidates(limits.max_cells)?;
        if candidates.is_empty() {
            let traced =
                super::graph::validate_collection_roots(self, roots, limits.max_graph_nodes)?;
            self.note_traced_graph(traced);
            increment(&self.0.collection.completed, 1);
            return Ok(0);
        }
        let (reachable, traced) =
            super::graph::reachable_cells(self, roots, limits.max_graph_nodes)?;
        let mut unreachable = Vec::new();
        unreachable
            .try_reserve(candidates.len())
            .map_err(|_| OwnershipError::ScratchUnavailable)?;
        for cell in candidates {
            if !reachable.contains(&cell.identity()) {
                unreachable.push(cell);
            }
        }
        // All candidates are pinned until every payload has been detached and released.
        let commit = commit::clear(self, &unreachable)?;
        let count = commit.cleared;
        drop(unreachable);
        let _ = self.registered_count();
        // Only completed passes scale later thresholds; a failed pass keeps counting.
        self.note_traced_graph(traced);
        increment(&self.0.collection.completed, 1);
        commit.finish();
        Ok(count)
    }
    fn collection_candidates(&self, limit: usize) -> Result<Vec<OwnedCell>, OwnershipError> {
        let mut registry = self.0.cells.lock().map_err(|_| OwnershipError::Poisoned)?;
        registry.retain(|_, cell| cell.strong_count() != 0);
        if registry.len() > limit {
            return Err(OwnershipError::ScratchUnavailable);
        }
        let mut candidates = Vec::new();
        candidates
            .try_reserve(registry.len())
            .map_err(|_| OwnershipError::ScratchUnavailable)?;
        for cell in registry.values() {
            if let Some(cell) = cell.upgrade() {
                candidates.push(OwnedCell(cell));
            }
        }
        Ok(candidates)
    }
}
