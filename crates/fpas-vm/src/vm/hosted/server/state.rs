//! Shared lifetime ownership, independent of a running VM worker.

use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use crate::vm::{TaskScheduler, cancellation::OwnedCancellation, hosted::HostedState};

/// Serialized stop deadline and completion state shared with the watchdog.
pub(super) struct Progress {
    pub(super) deadline: Option<Instant>,
    pub(super) finished: bool,
    effects_finished: bool,
}

/// Owns the work group and listeners without retaining the VM itself.
pub(super) struct Lifetime {
    pub(super) group: u64,
    pub(super) owner: u64,
    pub(super) cancellation: OwnedCancellation,
    pub(super) grace: Duration,
    pub(super) progress: Mutex<Progress>,
    pub(super) scheduler: Weak<TaskScheduler>,
    pub(super) hosted: Weak<HostedState>,
    listeners: Mutex<Vec<u64>>,
    errors: Mutex<Vec<String>>,
}

impl Lifetime {
    /// Construct a ready lifetime around an already registered task group.
    pub(super) fn new(
        group: u64,
        owner: u64,
        cancellation: OwnedCancellation,
        grace: Duration,
        scheduler: &Arc<TaskScheduler>,
        hosted: &Arc<HostedState>,
    ) -> Self {
        Self {
            group,
            owner,
            cancellation,
            grace,
            progress: Mutex::new(Progress {
                deadline: None,
                finished: false,
                effects_finished: false,
            }),
            scheduler: Arc::downgrade(scheduler),
            hosted: Arc::downgrade(hosted),
            listeners: Mutex::new(vec![]),
            errors: Mutex::new(vec![]),
        }
    }

    /// Whether the lifetime still admits resources and work.
    pub(super) fn ready(&self) -> bool {
        let state = self.progress.lock().unwrap_or_else(|e| e.into_inner());
        state.deadline.is_none() && !state.finished
    }

    /// Return the remaining common grace budget, rounded up to milliseconds.
    pub(super) fn remaining(&self) -> i64 {
        let state = self.progress.lock().unwrap_or_else(|e| e.into_inner());
        let remaining = state
            .deadline
            .map_or(self.grace, |d| d.saturating_duration_since(Instant::now()));
        // Round up so passing the budget to a millisecond wait does not expire early.
        remaining.as_micros().div_ceil(1000) as i64
    }

    /// Seal admission and close listeners once, preserving the original deadline.
    pub(super) fn request_stop(&self) -> bool {
        {
            let mut state = self.progress.lock().unwrap_or_else(|e| e.into_inner());
            if state.deadline.is_some() || state.finished {
                return false;
            }
            state.deadline = Some(Instant::now() + self.grace);
            if let Some(scheduler) = self.scheduler.upgrade() {
                // Admission is sealed before readiness can be observed as false.
                let _ = scheduler.groups.cancel(self.group);
            }
        }
        // Arm the independent watchdog before touching any other resource lock.
        self.cancellation.cancel();
        let listeners =
            std::mem::take(&mut *self.listeners.lock().unwrap_or_else(|e| e.into_inner()));
        if let Some(hosted) = self.hosted.upgrade() {
            for listener in listeners {
                if let Err(error) = hosted.network_listeners.close(listener) {
                    self.errors
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(error);
                }
            }
        }
        self.progress
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .effects_finished = true;
        true
    }

    /// Transfer a validated listener to this lifetime while it is ready.
    pub(super) fn own_listener(&self, listener: u64) -> Result<bool, String> {
        let mut listeners = self.listeners.lock().unwrap_or_else(|e| e.into_inner());
        if !self.ready() {
            return Err("Server is stopping; no new listener can be admitted".into());
        }
        if listeners.contains(&listener) {
            return Ok(false);
        }
        if listeners.len() >= 32 {
            return Err("A server lifetime may own at most 32 listeners".into());
        }
        let hosted = self.hosted.upgrade().ok_or("Server VM is closed")?;
        hosted.network_listeners.validate(listener)?;
        listeners.push(listener);
        Ok(true)
    }

    /// Snapshot retained listener-cleanup failures.
    pub(super) fn errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Check ownership before allowing a cross-lifetime listener transfer.
    pub(super) fn owns_listener(&self, listener: u64) -> bool {
        self.listeners
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&listener)
    }

    /// Disarm escalation after the owner confirms cleanup and closes the group.
    pub(super) fn finish(&self, caller: u64) -> Result<bool, String> {
        if caller != self.owner {
            return Err("Only the creating task may finish a server lifetime".into());
        }
        let mut state = self.progress.lock().unwrap_or_else(|e| e.into_inner());
        if state.finished {
            return Ok(true);
        }
        if state.deadline.is_none() {
            return Err("Call RequestStop before finishing shutdown".into());
        }
        if !state.effects_finished {
            return Err("Shutdown incomplete: listener closure is still in progress".into());
        }
        if let Some(scheduler) = self.scheduler.upgrade()
            && scheduler.groups.is_live(self.group)
        {
            return Err("Shutdown incomplete: close the server task group and retain its failure reports before FinishShutdown".into());
        }
        state.finished = true;
        Ok(true)
    }
}
