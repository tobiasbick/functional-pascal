//! Optional task ownership catalog and cooperative observation.

mod collection;
mod inspection;
mod lifecycle;
mod saved;
mod selection;

use super::{FailureSnapshot, Lease, identity::Candidate};
use crate::FunctionId;
use crate::execution::vm::{tasks::TaskState, worker::Worker};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
    },
};

/// Scheduler storage with optional synchronized task inspection ownership.
pub(in crate::execution::vm) use saved::ScheduledTask;

/// One task's current owner; saved storage contains the scheduler's original state.
pub(in crate::execution::vm) struct TaskRecord {
    id: u64,
    /// Original task entry used by fault snapshots after scheduler-result release.
    pub(in crate::execution::vm::failure_capture) function: FunctionId,
    state: AtomicU8,
    requested: AtomicBool,
    saved: Mutex<Option<TaskState>>,
    response: Mutex<Option<Arc<FailureSnapshot>>>,
    failure: Mutex<std::sync::Weak<Candidate>>,
    _lease: Lease,
}

/// Bounded metadata only; released results are removed rather than archived.
#[derive(Default)]
pub(super) struct Catalog {
    entries: Mutex<BTreeMap<u64, Arc<TaskRecord>>>,
    template: Mutex<Option<Worker>>,
    changed: Condvar,
    signal: Mutex<()>,
    // Counts published changes, so a collector never sleeps through one made during its scan.
    generation: AtomicU64,
    collecting: AtomicBool,
}

impl Catalog {
    /// Wake a waiting collector after a response or task state has been published.
    fn notify_changed(&self) {
        self.generation.fetch_add(1, Ordering::Release);
        // Serialize with a collector between its generation check and its wait.
        drop(self.signal.lock().unwrap_or_else(|p| p.into_inner()));
        self.changed.notify_all();
    }
}

const RUNNABLE: u8 = 0;
const RUNNING: u8 = 1;
const WAITING: u8 = 2;
const SLEEPING: u8 = 3;
const COMPLETED: u8 = 4;
const FAILED: u8 = 5;
const CANCELLED: u8 = 6;
/// Maximum number of retained task catalog entries.
pub(super) const MAX_TASKS: usize = 1024;

fn state_name(state: u8) -> &'static str {
    match state {
        RUNNING => "running",
        WAITING => "waiting",
        SLEEPING => "sleeping",
        COMPLETED => "completed",
        FAILED => "failed",
        CANCELLED => "cancelled",
        _ => "runnable",
    }
}
