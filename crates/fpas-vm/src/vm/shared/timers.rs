//! Cooperative timer queue for suspended spawned tasks.
//!
//! **Documentation:** `docs/pascal/language/concurrency/scheduling.md`.

use super::TaskState;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

/// Millisecond-bucketed task timer queue driven by one runtime thread.
pub(crate) struct TaskTimers {
    origin: Instant,
    sleeping: Mutex<BTreeMap<u64, Vec<TaskState>>>,
    changed: Condvar,
}

impl TaskTimers {
    /// Create an empty timer queue anchored to the current monotonic instant.
    pub(crate) fn new() -> Self {
        Self {
            origin: Instant::now(),
            sleeping: Mutex::new(BTreeMap::new()),
            changed: Condvar::new(),
        }
    }

    /// Suspend `task` until at least `milliseconds` have elapsed.
    pub(crate) fn schedule(&self, task: TaskState, milliseconds: u64) {
        let wake_millis = self.now_millis_ceil().saturating_add(milliseconds);
        let mut sleeping = self.sleeping.lock().unwrap_or_else(|e| e.into_inner());
        let previous_first = sleeping.first_key_value().map(|(&deadline, _)| deadline);
        sleeping.entry(wake_millis).or_default().push(task);
        if previous_first.is_none_or(|deadline| wake_millis < deadline) {
            self.changed.notify_one();
        }
    }

    /// Wait for and remove the next due millisecond bucket.
    pub(crate) fn wait_for_due(&self, shutdown: &AtomicBool) -> Option<Vec<TaskState>> {
        let mut sleeping = self.sleeping.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if shutdown.load(Ordering::Acquire) {
                sleeping.clear();
                return None;
            }

            let Some((&deadline, _)) = sleeping.first_key_value() else {
                sleeping = self
                    .changed
                    .wait(sleeping)
                    .unwrap_or_else(|e| e.into_inner());
                continue;
            };

            let now = self.now_millis_floor();
            if deadline <= now {
                return sleeping.remove(&deadline);
            }

            let timeout = Duration::from_millis(deadline - now);
            let (guard, _) = self
                .changed
                .wait_timeout(sleeping, timeout)
                .unwrap_or_else(|e| e.into_inner());
            sleeping = guard;
        }
    }

    /// Wake the timer driver so it can observe runtime shutdown.
    pub(crate) fn notify_shutdown(&self) {
        self.changed.notify_all();
    }

    fn now_millis_floor(&self) -> u64 {
        self.origin.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
    }

    fn now_millis_ceil(&self) -> u64 {
        let nanos = self.origin.elapsed().as_nanos();
        nanos.div_ceil(1_000_000).min(u128::from(u64::MAX)) as u64
    }
}
