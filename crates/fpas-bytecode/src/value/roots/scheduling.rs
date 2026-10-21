//! Per-domain pressure scheduling with configurable heuristic defaults.

use super::super::MemoryDomain;
use std::num::NonZeroUsize;
use std::sync::atomic::Ordering;

/// Collection thresholds between successful passes, with heuristic initial defaults.
///
/// Counts apply between successful whole-domain passes. External root releases and
/// quiescent edge-removing mutations bypass these running-execution thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CollectionSchedule {
    /// Cell allocations before requesting a pass.
    pub allocations: NonZeroUsize,
    /// Successful cell mutations before requesting a pass during execution.
    pub mutations: NonZeroUsize,
    /// Completed execution batches before a pass, even without new allocations.
    pub safe_points: NonZeroUsize,
}
impl Default for CollectionSchedule {
    #[expect(clippy::expect_used, reason = "approved literal defaults are nonzero")]
    fn default() -> Self {
        Self {
            allocations: NonZeroUsize::new(4096).expect("nonzero cell threshold"),
            mutations: NonZeroUsize::new(8192).expect("nonzero mutation threshold"),
            safe_points: NonZeroUsize::new(256).expect("nonzero batch interval"),
        }
    }
}

pub(super) struct Pressure {
    pub schedule: CollectionSchedule,
    allocations: usize,
    mutations: usize,
    safe_points: usize,
}
impl Pressure {
    pub(super) fn new(schedule: CollectionSchedule) -> Self {
        Self {
            schedule,
            allocations: 0,
            mutations: 0,
            safe_points: 0,
        }
    }
    pub(super) fn reset(&mut self) {
        self.allocations = 0;
        self.mutations = 0;
        self.safe_points = 0;
    }
}

enum Event {
    Allocation,
    Mutation,
    SafePoint,
}
impl MemoryDomain {
    /// Effective scheduled policy; other maintenance policies have no pressure thresholds.
    #[must_use]
    pub fn collection_schedule(&self) -> Option<CollectionSchedule> {
        self.0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pressure
            .as_ref()
            .map(|pressure| pressure.schedule)
    }
    pub(in crate::value) fn note_cell_allocation(&self) {
        if self.0.roots.scheduled.load(Ordering::Acquire) {
            self.note_pressure(Event::Allocation);
        }
    }
    pub(in crate::value) fn note_cell_mutation(&self) {
        if self.0.roots.scheduled.load(Ordering::Acquire) {
            self.note_pressure(Event::Mutation);
        } else {
            self.request_maintenance();
        }
    }
    pub(super) fn note_safe_point(&self) {
        if self.0.roots.scheduled.load(Ordering::Acquire) {
            self.note_pressure(Event::SafePoint);
        }
    }
    fn note_pressure(&self, event: Event) {
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let quiescent_mutation = matches!(event, Event::Mutation) && state.execution == 0;
        let Some(pressure) = &mut state.pressure else {
            return;
        };
        let (count, threshold) = match event {
            Event::Allocation => (&mut pressure.allocations, pressure.schedule.allocations),
            Event::Mutation => (&mut pressure.mutations, pressure.schedule.mutations),
            Event::SafePoint => (&mut pressure.safe_points, pressure.schedule.safe_points),
        };
        let crossed = *count < threshold.get();
        *count = count.saturating_add(1);
        let request = (crossed && *count >= threshold.get()) || quiescent_mutation;
        drop(state);
        if request {
            self.request_maintenance();
        }
    }

    /// Request scheduled collection before parking or after saved-state publication.
    /// Complete roots must already be published; opaque execution still defers service.
    /// Explicit-only domains ignore this notification. Running thresholds never suppress it.
    pub fn request_collection_at_rest(&self) {
        if self.0.roots.scheduled.load(Ordering::Acquire) {
            self.request_maintenance();
        }
    }
}

#[cfg(test)]
mod tests;
