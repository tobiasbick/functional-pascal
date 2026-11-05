//! Optional execution-wide resource budgets shared by tasks and callbacks.

mod heap;
mod output;
#[cfg(test)]
mod tests;

pub(crate) use heap::shared_size;
pub(crate) use heap::{Reservation, Scope, catch, current_budget, heap_active, output, preflight};
pub(crate) use output::format_output;

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

/// Optional bounds for one execution, including every spawned task and callback.
///
/// Heap bytes cover managed value storage, not process RSS or child processes.
/// A deadline is cooperative at VM and supported host-operation checkpoints.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionLimits {
    /// Maximum simultaneously retained managed storage bytes.
    pub max_heap_bytes: Option<usize>,
    /// Maximum instructions dispatched across all execution workers.
    pub max_instructions: Option<u64>,
    /// Maximum active execution time; debugger pauses do not consume this time.
    pub timeout: Option<Duration>,
    /// Maximum logical console-output bytes, including line endings.
    pub max_output_bytes: Option<usize>,
}

/// Resource observations for one execution; shared allocations are counted once.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ExecutionUsage {
    /// Currently retained managed storage bytes.
    pub heap_bytes: usize,
    /// Largest simultaneous managed storage reservation.
    pub peak_heap_bytes: usize,
    /// Instructions admitted under an instruction budget.
    pub instructions: u64,
    /// Logical console bytes admitted under an output budget.
    pub output_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Exhausted {
    Heap { limit: usize, requested: usize },
    Instructions { limit: u64 },
    Timeout,
    Output { limit: usize },
}

impl std::fmt::Display for Exhausted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Heap { limit, requested } => write!(
                f,
                "Execution heap limit of {limit} bytes exceeded (requested {requested} additional bytes)"
            ),
            Self::Instructions { limit } => {
                write!(f, "Execution instruction limit of {limit} exceeded")
            }
            Self::Timeout => f.write_str("Execution deadline exceeded"),
            Self::Output { limit } => write!(f, "Execution output limit of {limit} bytes exceeded"),
        }
    }
}

/// Shared accounting state; no value graph or VM is retained here.
#[derive(Debug, Default)]
pub(crate) struct Budget {
    limits: RwLock<ExecutionLimits>,
    pub(crate) heap_limit: AtomicUsize,
    heap_enabled: AtomicBool,
    instruction_limit: AtomicU64,
    instruction_enabled: AtomicBool,
    output_limit: AtomicUsize,
    output_enabled: AtomicBool,
    timeout_nanos: AtomicU64,
    elapsed_nanos: AtomicU64,
    active_since: Mutex<Option<Instant>>,
    heap: AtomicUsize,
    peak: AtomicUsize,
    instructions: AtomicU64,
    output: AtomicUsize,
    failed: AtomicBool,
    failure: Mutex<Option<Exhausted>>,
    cleanup_started: OnceLock<Instant>,
    cleanup_instructions: AtomicU64,
}

impl Budget {
    pub(crate) fn configure(&self, limits: ExecutionLimits) {
        *self.limits.write().unwrap_or_else(|p| p.into_inner()) = limits;
        self.heap_limit.store(
            limits.max_heap_bytes.unwrap_or(usize::MAX),
            Ordering::Relaxed,
        );
        self.heap_enabled
            .store(limits.max_heap_bytes.is_some(), Ordering::Relaxed);
        self.instruction_limit.store(
            limits.max_instructions.unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
        self.instruction_enabled
            .store(limits.max_instructions.is_some(), Ordering::Relaxed);
        self.output_limit.store(
            limits.max_output_bytes.unwrap_or(usize::MAX),
            Ordering::Relaxed,
        );
        self.output_enabled
            .store(limits.max_output_bytes.is_some(), Ordering::Relaxed);
        self.timeout_nanos.store(
            limits.timeout.map_or(0, |d| {
                u64::try_from(d.as_nanos()).unwrap_or(u64::MAX).max(1)
            }),
            Ordering::Relaxed,
        );
    }

    pub(crate) fn limits(&self) -> ExecutionLimits {
        *self.limits.read().unwrap_or_else(|p| p.into_inner())
    }

    pub(crate) fn tracks_heap(&self) -> bool {
        self.heap_enabled.load(Ordering::Relaxed)
    }

    pub(crate) fn usage(&self) -> ExecutionUsage {
        ExecutionUsage {
            heap_bytes: self.heap.load(Ordering::Relaxed),
            peak_heap_bytes: self.peak.load(Ordering::Relaxed),
            instructions: self.instructions.load(Ordering::Relaxed),
            output_bytes: self.output.load(Ordering::Relaxed),
        }
    }

    pub(crate) fn fail(&self, error: Exhausted) -> Exhausted {
        let mut failure = self.failure.lock().unwrap_or_else(|p| p.into_inner());
        let first = *failure.get_or_insert(error);
        self.failed.store(true, Ordering::Release);
        first
    }

    pub(crate) fn failure(&self) -> Option<Exhausted> {
        self.failed
            .load(Ordering::Acquire)
            .then(|| *self.failure.lock().unwrap_or_else(|p| p.into_inner()))
            .flatten()
    }

    pub(crate) fn resume(&self) {
        if self.timeout_nanos.load(Ordering::Relaxed) != 0 {
            *self.active_since.lock().unwrap_or_else(|p| p.into_inner()) = Some(Instant::now());
        }
    }

    pub(crate) fn pause(&self) {
        if let Some(since) = self
            .active_since
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
        {
            self.elapsed_nanos.fetch_add(
                u64::try_from(since.elapsed().as_nanos()).unwrap_or(u64::MAX),
                Ordering::Relaxed,
            );
        }
    }

    pub(crate) fn checkpoint(&self) -> Result<(), Exhausted> {
        if let Some(failure) = self.failure() {
            return Err(failure);
        }
        let timeout = self.timeout_nanos.load(Ordering::Relaxed);
        if timeout != 0 {
            let active = self.active_since.lock().unwrap_or_else(|p| p.into_inner());
            let elapsed = active.map_or(0, |start| {
                u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
            });
            if self
                .elapsed_nanos
                .load(Ordering::Relaxed)
                .saturating_add(elapsed)
                >= timeout
            {
                return Err(self.fail(Exhausted::Timeout));
            }
        }
        Ok(())
    }

    pub(crate) fn cleanup_checkpoint(&self, cleanup: bool) -> Result<(), Exhausted> {
        if cleanup && let Some(error) = self.failure() {
            if self.cleanup_started.get_or_init(Instant::now).elapsed() >= Duration::from_secs(1) {
                return Err(error);
            }
            return Ok(());
        }
        self.checkpoint()
    }

    pub(crate) fn instruction(&self, cleanup: bool) -> Result<(), Exhausted> {
        if let Some(error) = self.failure() {
            if !cleanup || self.cleanup_instructions.fetch_add(1, Ordering::Relaxed) >= 100_000 {
                return Err(error);
            }
            return self.cleanup_checkpoint(true);
        }
        if self.instruction_enabled.load(Ordering::Relaxed) {
            let limit = self.instruction_limit.load(Ordering::Relaxed);
            if self
                .instructions
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                    (n < limit).then(|| n + 1)
                })
                .is_err()
            {
                return Err(self.fail(Exhausted::Instructions { limit }));
            }
        }
        Ok(())
    }

    pub(crate) fn output(&self, bytes: usize) -> Result<(), Exhausted> {
        if self.output_enabled.load(Ordering::Relaxed) {
            let limit = self.output_limit.load(Ordering::Relaxed);
            if self
                .output
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                    n.checked_add(bytes).filter(|n| *n <= limit)
                })
                .is_err()
            {
                return Err(self.fail(Exhausted::Output { limit }));
            }
        }
        Ok(())
    }

    fn check_output(&self, bytes: usize) -> Result<(), Exhausted> {
        let limit = self.output_limit.load(Ordering::Relaxed);
        if self
            .output
            .load(Ordering::Relaxed)
            .checked_add(bytes)
            .is_none_or(|n| n > limit)
        {
            return Err(self.fail(Exhausted::Output { limit }));
        }
        Ok(())
    }

    fn reserve(&self, bytes: usize) -> Result<(), Exhausted> {
        let limit = self.effective_heap_limit();
        let previous = self
            .heap
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                n.checked_add(bytes).filter(|n| *n <= limit)
            })
            .map_err(|_| {
                self.fail(Exhausted::Heap {
                    limit,
                    requested: bytes,
                })
            })?;
        self.peak.fetch_max(previous + bytes, Ordering::Relaxed);
        Ok(())
    }

    fn effective_heap_limit(&self) -> usize {
        let limit = self.heap_limit.load(Ordering::Relaxed);
        if self.failed.load(Ordering::Acquire) {
            limit.saturating_add(64 * 1024)
        } else {
            limit
        }
    }
}
