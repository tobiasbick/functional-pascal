//! Opt-in sampling and non-overlapping timing of nested inspection operations.

use std::cell::Cell;

use fpas_bytecode::profile::{self, Counter, Timer};

use super::DebugSession;

impl DebugSession {
    /// Enable routine and source-line sampling of this debug target.
    /// The recorder must describe the same executable; image replacement is disabled.
    #[must_use]
    pub fn with_profile(mut self, profile: crate::vm::DebugProfile) -> Self {
        if let Some(worker) = self.runtime.worker(0) {
            let _ = worker.hosted.profile.set(profile.sink());
        }
        self.profile = Some(profile);
        self
    }
}

thread_local! {
    static INSPECTING: Cell<bool> = const { Cell::new(false) };
}

pub(super) struct InspectionTimer(Option<Timer>);

/// Snapshot capture and evaluation can call other inspection APIs on the same thread.
pub(super) fn inspection() -> InspectionTimer {
    InspectionTimer(
        (profile::enabled() && !INSPECTING.replace(true))
            .then(|| Timer::start(Counter::DebugInspectionNanos)),
    )
}

impl Drop for InspectionTimer {
    fn drop(&mut self) {
        if self.0.take().is_some() {
            INSPECTING.set(false);
        }
    }
}
