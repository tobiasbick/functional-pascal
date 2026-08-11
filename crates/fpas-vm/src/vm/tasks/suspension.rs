//! Explicit cooperative suspension state used by the deterministic debugger driver.

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use fpas_bytecode::Register;

enum DebugClockMode {
    Realtime(Instant),
    #[cfg(test)]
    Manual(AtomicU64),
}

/// Monotonic clock used only by the debugger's deterministic execution lane.
pub(in crate::vm) struct DebugClock {
    mode: DebugClockMode,
}

impl DebugClock {
    /// Create a clock backed by host monotonic time.
    pub(in crate::vm) fn realtime() -> Self {
        Self {
            mode: DebugClockMode::Realtime(Instant::now()),
        }
    }

    /// Create a manually advanced clock for deterministic scheduler tests.
    #[cfg(test)]
    pub(in crate::vm) fn manual() -> Self {
        Self {
            mode: DebugClockMode::Manual(AtomicU64::new(0)),
        }
    }

    /// Return elapsed debugger-clock milliseconds.
    pub(super) fn now_millis(&self) -> u64 {
        match &self.mode {
            DebugClockMode::Realtime(origin) => {
                origin.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
            }
            #[cfg(test)]
            DebugClockMode::Manual(now) => now.load(Ordering::Acquire),
        }
    }

    /// Wait or deterministically advance by the requested duration.
    pub(in crate::vm) fn wait(&self, duration: Duration) {
        match &self.mode {
            DebugClockMode::Realtime(_) => std::thread::sleep(duration),
            #[cfg(test)]
            DebugClockMode::Manual(now) => {
                let milliseconds = duration.as_millis().max(1).min(u128::from(u64::MAX)) as u64;
                now.fetch_add(milliseconds, Ordering::AcqRel);
            }
        }
    }
}

/// Work that must complete before a cooperatively debugged task is runnable again.
pub(in crate::vm) enum TaskSuspension {
    /// Resume after giving another runnable task a scheduling turn.
    Yield,
    /// Resume after one retained task result becomes available.
    Wait {
        id: u64,
        destination: Option<Register>,
    },
    /// Resume after all retained task results become available.
    WaitAll { ids: Vec<u64> },
    /// Resume after the debugger-clock deadline is reached.
    Sleep { deadline_millis: u64 },
}

impl TaskSuspension {
    /// Construct a sleep deadline relative to the supplied debugger clock.
    pub(super) fn sleep(milliseconds: u64, clock: &DebugClock) -> Self {
        Self::Sleep {
            deadline_millis: clock.now_millis().saturating_add(milliseconds),
        }
    }

    /// Return the current scheduler-visible state using the supplied clock.
    pub(in crate::vm) fn state(&self, clock: &DebugClock) -> TaskSuspensionState {
        match self {
            Self::Yield => TaskSuspensionState::Yielded,
            Self::Wait { .. } | Self::WaitAll { .. } => TaskSuspensionState::Waiting,
            Self::Sleep { deadline_millis } => TaskSuspensionState::Sleeping {
                remaining: Duration::from_millis(
                    deadline_millis.saturating_sub(clock.now_millis()),
                ),
            },
        }
    }
}

/// Scheduler-facing readiness of one cooperatively suspended task.
pub(in crate::vm) enum TaskSuspensionState {
    /// The task yielded and can be scheduled immediately.
    Yielded,
    /// The task waits for one or more retained results.
    Waiting,
    /// The task waits for a monotonic timer.
    Sleeping { remaining: Duration },
}
