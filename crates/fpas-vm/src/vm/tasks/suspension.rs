//! Shared cooperative suspension state for the worker pool and deterministic debugger.

#[cfg(test)]
mod clock_tests;

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use fpas_bytecode::{Register, Value};

enum TaskClockMode {
    Realtime(Instant),
    #[cfg(test)]
    Manual(AtomicU64),
}

/// Monotonic task clock, manually advanced only by deterministic debugger tests.
pub(in crate::vm) struct TaskClock {
    mode: TaskClockMode,
}

impl TaskClock {
    /// Create a clock backed by host monotonic time.
    pub(in crate::vm) fn realtime() -> Self {
        Self {
            mode: TaskClockMode::Realtime(Instant::now()),
        }
    }

    /// Create a manually advanced clock for deterministic scheduler tests.
    #[cfg(test)]
    pub(in crate::vm) fn manual() -> Self {
        Self {
            mode: TaskClockMode::Manual(AtomicU64::new(0)),
        }
    }

    /// Return elapsed monotonic milliseconds.
    pub(super) fn now_millis(&self) -> u64 {
        match &self.mode {
            TaskClockMode::Realtime(origin) => {
                origin.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
            }
            #[cfg(test)]
            TaskClockMode::Manual(now) => now.load(Ordering::Acquire),
        }
    }

    /// Round realtime deadlines upward so a relative wait never expires before its budget.
    pub(super) fn deadline_after(&self, duration: Duration) -> u64 {
        if duration.is_zero() {
            return self.now_millis();
        }
        let elapsed_nanos = match &self.mode {
            TaskClockMode::Realtime(origin) => origin.elapsed().as_nanos(),
            #[cfg(test)]
            TaskClockMode::Manual(now) => u128::from(now.load(Ordering::Acquire)) * 1_000_000,
        };
        elapsed_nanos
            .saturating_add(duration.as_nanos())
            .div_ceil(1_000_000)
            .min(u128::from(u64::MAX)) as u64
    }

    /// Wait or deterministically advance by the requested duration.
    pub(in crate::vm) fn wait(&self, duration: Duration) {
        match &self.mode {
            TaskClockMode::Realtime(_) => std::thread::sleep(duration),
            #[cfg(test)]
            TaskClockMode::Manual(now) => {
                let milliseconds = duration.as_millis().max(1).min(u128::from(u64::MAX)) as u64;
                now.fetch_add(milliseconds, Ordering::AcqRel);
            }
        }
    }
}

/// Work that must complete before a cooperatively suspended task is runnable again.
pub(in crate::vm) enum TaskSuspension {
    /// Retried worker admission waits for a cancellable debugger-clock delay.
    SupervisionBackoff { deadline_millis: u64 },
    /// Retain a sealed group until all its children terminate.
    GroupClose {
        id: u64,
        deadline_millis: Option<u64>,
        destination: Option<Register>,
    },
    /// Own mixed selection cases until one operation commits.
    Selection(Box<super::selection::SelectionWait>),
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
    /// Resume on task completion, cancellation, or a monotonic task-clock deadline.
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
    pub(super) fn sleep(milliseconds: u64, clock: &TaskClock) -> Self {
        Self::Sleep {
            deadline_millis: clock.deadline_after(Duration::from_millis(milliseconds)),
        }
    }

    /// Return the current scheduler-visible state using the supplied clock.
    pub(in crate::vm) fn state(&self, clock: &TaskClock) -> TaskSuspensionState {
        match self {
            Self::GroupClose {
                deadline_millis: None,
                ..
            } => TaskSuspensionState::Waiting,
            Self::Selection(wait) => wait.debug_state(clock),
            Self::GroupClose {
                deadline_millis: Some(deadline),
                ..
            }
            | Self::WaitAnyControlled {
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
            Self::Sleep { deadline_millis } | Self::SupervisionBackoff { deadline_millis } => {
                TaskSuspensionState::Sleeping {
                    remaining: Duration::from_millis(
                        deadline_millis.saturating_sub(clock.now_millis()),
                    ),
                }
            }
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
