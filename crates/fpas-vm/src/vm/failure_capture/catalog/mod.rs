//! Optional task ownership catalog and cooperative observation.

mod collection;
mod inspection;
mod lifecycle;
mod saved;
mod selection;

use super::{FailureSnapshot, Lease, identity::Candidate};
use crate::vm::{tasks::TaskState, worker::Worker};
use fpas_bytecode::FunctionId;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU8},
    },
};

/// Scheduler storage with optional synchronized task inspection ownership.
pub(in crate::vm) use saved::ScheduledTask;

/// One task's current owner; saved storage contains the scheduler's original state.
pub(in crate::vm) struct TaskRecord {
    id: u64,
    /// Original task entry used by fault snapshots after scheduler-result release.
    pub(in crate::vm::failure_capture) function: FunctionId,
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
    collecting: AtomicBool,
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
