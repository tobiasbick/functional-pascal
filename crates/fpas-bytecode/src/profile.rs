//! Process-wide runtime event counters for `fpas run --profile`.
//!
//! Counting is off by default; an event site then costs one relaxed load. Enabled counts go
//! to one of several cache-line-aligned stripes chosen per thread, so pool workers do not
//! contend on a shared counter.
//!
//! Documentation: `docs/pascal/tools/profiler.md`.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

/// One counted runtime event or duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counter {
    /// Ownership classification walks.
    ClassifyCalls,
    /// Allocations visited by ownership classification walks.
    ClassifyVisited,
    /// Serviced collection passes.
    CollectionPasses,
    /// Wall time of serviced collection passes.
    CollectionNanos,
    /// Root values traced by collection passes.
    CollectionVisited,
    /// Root publication and safe points after dispatch batches.
    SafePointNanos,
    /// Spawned tasks.
    TaskSpawns,
    /// Tasks that gave up their time slice to queued work.
    TaskYields,
    /// Cooperative task suspensions on waits, sleeps and channel operations.
    TaskParks,
    /// Scheduler-queue lock acquisitions that had to wait.
    QueueContended,
    /// Time pool workers waited for runnable tasks.
    WorkerIdleNanos,
    /// Cancellation registry lookups from `IsCancellationRequested`.
    CancellationLookups,
}

impl Counter {
    /// Every counter, in report order.
    pub const ALL: [Self; 12] = [
        Self::ClassifyCalls,
        Self::ClassifyVisited,
        Self::CollectionPasses,
        Self::CollectionNanos,
        Self::CollectionVisited,
        Self::SafePointNanos,
        Self::TaskSpawns,
        Self::TaskYields,
        Self::TaskParks,
        Self::QueueContended,
        Self::WorkerIdleNanos,
        Self::CancellationLookups,
    ];

    /// Stable report field name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ClassifyCalls => "classify_calls",
            Self::ClassifyVisited => "classify_visited",
            Self::CollectionPasses => "collection_passes",
            Self::CollectionNanos => "collection_nanos",
            Self::CollectionVisited => "collection_visited",
            Self::SafePointNanos => "safe_point_nanos",
            Self::TaskSpawns => "task_spawns",
            Self::TaskYields => "task_yields",
            Self::TaskParks => "task_parks",
            Self::QueueContended => "queue_contended",
            Self::WorkerIdleNanos => "worker_idle_nanos",
            Self::CancellationLookups => "cancellation_lookups",
        }
    }
}

const COUNTERS: usize = Counter::ALL.len();
const STRIPES: usize = 32;

#[repr(align(128))]
struct Stripe([AtomicU64; COUNTERS]);

static ENABLED: AtomicBool = AtomicBool::new(false);
static NEXT_STRIPE: AtomicUsize = AtomicUsize::new(0);
static STRIPES_STORAGE: [Stripe; STRIPES] =
    [const { Stripe([const { AtomicU64::new(0) }; COUNTERS]) }; STRIPES];

thread_local! {
    static STRIPE: usize = NEXT_STRIPE.fetch_add(1, Ordering::Relaxed) % STRIPES;
}

/// Profiled runs currently counting, and whether any of them overlapped.
static RUNS: Mutex<(usize, bool)> = Mutex::new((0, false));

/// One profiled run's claim on the process-wide counters.
///
/// The first concurrent run resets the counters. Overlapping runs share them, so their
/// snapshots report `exclusive: false`.
#[must_use]
pub struct Run {
    finished: bool,
}

/// Counter totals of one finished run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Every counter, in report order.
    pub counts: Vec<(Counter, u64)>,
    /// No other profiled run counted at any time during this run.
    pub exclusive: bool,
}

/// Start counting for one profiled run.
pub fn begin() -> Run {
    let mut runs = RUNS.lock().unwrap_or_else(PoisonError::into_inner);
    if runs.0 == 0 {
        for stripe in &STRIPES_STORAGE {
            for count in &stripe.0 {
                count.store(0, Ordering::Relaxed);
            }
        }
        runs.1 = false;
    } else {
        runs.1 = true;
    }
    runs.0 += 1;
    ENABLED.store(true, Ordering::Release);
    Run { finished: false }
}

impl Run {
    /// Stop this run's counting and return the totals it observed.
    #[must_use]
    pub fn finish(mut self) -> Snapshot {
        let exclusive = self.release();
        Snapshot {
            counts: totals(),
            exclusive,
        }
    }

    fn release(&mut self) -> bool {
        self.finished = true;
        let mut runs = RUNS.lock().unwrap_or_else(PoisonError::into_inner);
        runs.0 -= 1;
        if runs.0 == 0 {
            ENABLED.store(false, Ordering::Release);
        }
        !runs.1
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        if !self.finished {
            self.release();
        }
    }
}

/// Whether a profiled run is counting events.
#[inline]
#[must_use]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Add `amount` to `counter` while counting is enabled.
#[inline]
pub fn add(counter: Counter, amount: u64) {
    if enabled() {
        add_enabled(counter, amount);
    }
}

#[cold]
fn add_enabled(counter: Counter, amount: u64) {
    let stripe = STRIPE.try_with(|stripe| *stripe).unwrap_or(0);
    STRIPES_STORAGE[stripe].0[counter as usize].fetch_add(amount, Ordering::Relaxed);
}

/// Adds the time until it is dropped to a duration counter; inert while not counting.
#[must_use]
pub struct Timer(Option<(Counter, std::time::Instant)>);

impl Timer {
    /// Start measuring into `counter` if counting is enabled.
    #[inline]
    pub fn start(counter: Counter) -> Self {
        Self(enabled().then(|| (counter, std::time::Instant::now())))
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some((counter, started)) = self.0 {
            add(
                counter,
                u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
            );
        }
    }
}

fn totals() -> Vec<(Counter, u64)> {
    Counter::ALL
        .iter()
        .map(|&counter| {
            let total = STRIPES_STORAGE
                .iter()
                .map(|stripe| stripe.0[counter as usize].load(Ordering::Relaxed))
                .fold(0u64, u64::saturating_add);
            (counter, total)
        })
        .collect()
}

#[cfg(test)]
mod tests;
