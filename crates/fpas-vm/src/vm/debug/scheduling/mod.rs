//! Controlled batch scheduling metadata and a shared pre-effect rejection latch.

mod effects;
pub(in crate::vm::debug) mod rounds;

use std::sync::Mutex;

use serde::Serialize;

use crate::vm::VmError;

/// Versioned dispatch, randomness and virtual-time policy.
pub const SCHEDULE_ALGORITHM: &str = "seeded-rounds-v1";

/// Bounded description of requested and observed scheduling, without a turn log.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DebugScheduleReport {
    /// Requested mode: default or seeded.
    pub mode: &'static str,
    /// Full-width decimal seed, independent of JSON number precision.
    pub seed: Option<String>,
    /// Version of the complete seeded execution model.
    pub algorithm: Option<&'static str>,
    /// Requested realtime or virtual task clock.
    pub clock: &'static str,
    /// Coverage of the actual execution, independent of program success.
    pub status: &'static str,
    /// Explicit reasons for rejected or unavailable execution.
    pub reasons: Vec<&'static str>,
    /// Begun rounds, including a partial final round.
    pub rounds: Option<String>,
    /// Driver dispatch attempts, distinct from packed-instruction counts.
    pub dispatches: Option<String>,
    /// Observed virtual task time in milliseconds.
    pub virtual_time_millis: Option<String>,
}

impl DebugScheduleReport {
    /// Describe a request before a worker has supplied authoritative counters.
    #[must_use]
    pub fn requested(seed: Option<u64>) -> Self {
        Self {
            mode: if seed.is_some() { "seeded" } else { "default" },
            seed: seed.map(|value| value.to_string()),
            algorithm: seed.map(|_| SCHEDULE_ALGORITHM),
            clock: if seed.is_some() {
                "virtual"
            } else {
                "realtime"
            },
            status: if seed.is_some() {
                "not_started"
            } else {
                "not_requested"
            },
            reasons: Vec::new(),
            rounds: None,
            dispatches: None,
            virtual_time_millis: None,
        }
    }
}

/// A forbidden effect must stop batch even when a task group contains its failure.
#[derive(Default)]
pub(in crate::vm) struct ScheduleGuard {
    rejection: Mutex<Option<(u64, VmError)>>,
}

impl ScheduleGuard {
    pub(in crate::vm) fn rejection(&self) -> Option<(u64, VmError)> {
        self.rejection
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn reject(&self, task: u64, error: VmError) -> VmError {
        let mut rejection = self.rejection.lock().unwrap_or_else(|e| e.into_inner());
        rejection.get_or_insert_with(|| (task, error.clone()));
        error
    }
}
