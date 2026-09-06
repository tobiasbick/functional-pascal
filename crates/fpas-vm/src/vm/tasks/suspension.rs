//! Explicit cooperative suspension state used by the deterministic debugger driver.

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use fpas_bytecode::{Register, Value};

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
    /// Resume when one retained task completes without consuming its result.
    WaitAny {
        ids: Vec<u64>,
        destination: Option<Register>,
    },
    /// Resume on task completion, cancellation, or a debugger-clock deadline.
    WaitAnyControlled {
        ids: Vec<u64>,
        token: Option<u64>,
        deadline_millis: Option<u64>,
        destination: Option<Register>,
    },
    /// Resume after a bounded channel accepts a value, closes, or is cancelled.
    ChannelSend {
        handle: u64,
        value: Value,
        token: Option<u64>,
        destination: Option<Register>,
    },
    /// Resume after a bounded channel yields a value, closes, or is cancelled.
    ChannelReceive {
        handle: u64,
        token: Option<u64>,
        destination: Option<Register>,
    },
    /// Resume when a channel accepts a value, closes, or reaches its deadline.
    ChannelSendTimeout {
        handle: u64,
        value: Value,
        deadline_millis: u64,
        destination: Option<Register>,
    },
    /// Resume when a channel yields a value, closes, or reaches its deadline.
    ChannelReceiveTimeout {
        handle: u64,
        deadline_millis: u64,
        destination: Option<Register>,
    },
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
            Self::WaitAnyControlled {
                deadline_millis: Some(deadline),
                ..
            } => TaskSuspensionState::Sleeping {
                remaining: Duration::from_millis(deadline.saturating_sub(clock.now_millis())),
            },
            Self::WaitAnyControlled {
                deadline_millis: None,
                ..
            } => TaskSuspensionState::Waiting,
            Self::Yield => TaskSuspensionState::Yielded,
            Self::Wait { .. }
            | Self::WaitAll { .. }
            | Self::WaitAny { .. }
            | Self::ChannelSend { .. }
            | Self::ChannelReceive { .. } => TaskSuspensionState::Waiting,
            Self::ChannelSendTimeout {
                deadline_millis, ..
            }
            | Self::ChannelReceiveTimeout {
                deadline_millis, ..
            } => TaskSuspensionState::Sleeping {
                remaining: Duration::from_millis(
                    deadline_millis.saturating_sub(clock.now_millis()),
                ),
            },
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
