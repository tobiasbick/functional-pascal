//! Per-domain pressure scheduling with configurable heuristic defaults.

use super::super::MemoryDomain;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// Event counts and thresholds between successful passes.
///
/// Atomic so the executing owner records events without the gate mutex. Every
/// recording thread holds the exclusive domain gate, which also excludes a pass,
/// so counting needs no read-modify-write.
#[derive(Default)]
pub(super) struct Pressure {
    /// Allocation, mutation and safe-point thresholds; zero while unscheduled.
    limits: [AtomicUsize; 3],
    counts: [AtomicUsize; 3],
}
impl Pressure {
    /// Install a schedule, or clear it, and restart counting. Callers hold the gate
    /// mutex or install before the domain is shared.
    pub(super) fn install(&self, schedule: Option<CollectionSchedule>) {
        let limits = schedule.map_or([0; 3], |schedule| {
            [
                schedule.allocations.get(),
                schedule.mutations.get(),
                schedule.safe_points.get(),
            ]
        });
        for (slot, limit) in self.limits.iter().zip(limits) {
            slot.store(limit, Ordering::Relaxed);
        }
        self.reset();
    }
    pub(super) fn reset(&self) {
        for count in &self.counts {
            count.store(0, Ordering::Relaxed);
        }
    }
    fn schedule(&self) -> Option<CollectionSchedule> {
        let [allocations, mutations, safe_points] = self
            .limits
            .each_ref()
            .map(|limit| NonZeroUsize::new(limit.load(Ordering::Relaxed)));
        Some(CollectionSchedule {
            allocations: allocations?,
            mutations: mutations?,
            safe_points: safe_points?,
        })
    }
}

#[derive(Clone, Copy)]
enum Event {
    Allocation,
    Mutation,
    SafePoint,
}
impl MemoryDomain {
    /// Effective scheduled policy; other maintenance policies have no pressure thresholds.
    #[must_use]
    pub fn collection_schedule(&self) -> Option<CollectionSchedule> {
        self.0.roots.pressure.schedule()
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
        let pressure = &self.0.roots.pressure;
        let limit = pressure.limits[event as usize].load(Ordering::Relaxed);
        if limit == 0 {
            return;
        }
        // The executing owner holds an active execution, so its mutations are never quiescent.
        let quiescent_mutation = matches!(event, Event::Mutation)
            && self.0.roots.exec_owner.load(Ordering::Acquire) != super::thread_token()
            && self
                .0
                .roots
                .access
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .execution
                == 0;
        let count = &pressure.counts[event as usize];
        let next = count.load(Ordering::Relaxed).saturating_add(1);
        count.store(next, Ordering::Relaxed);
        if next == limit || quiescent_mutation {
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
