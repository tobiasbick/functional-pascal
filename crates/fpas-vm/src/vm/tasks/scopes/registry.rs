//! Shared ownership, cancellation, and failure state of open task scopes.
//!
//! Documentation: `docs/pascal/language/concurrency/task-scopes.md`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::vm::VmError;

use super::failures::{ScopeFailure, is_cancellation};

/// Cancellation flags shared by a scope, its children, and the task running its body.
#[derive(Default)]
pub(in crate::vm) struct ScopeControl {
    cancelled: AtomicBool,
    failed: AtomicBool,
}

impl ScopeControl {
    /// Whether the scope's children must stop at their next boundary.
    pub(in crate::vm) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// Whether a child failed, so the scope body must stop as well.
    pub(in crate::vm) fn has_failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }

    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn fail(&self) {
        self.failed.store(true, Ordering::Release);
        self.cancel();
    }
}

struct Child {
    id: u64,
    done: bool,
}

struct Scope {
    control: Arc<ScopeControl>,
    children: Vec<Child>,
    failures: Vec<ScopeFailure>,
}

#[derive(Default)]
struct State {
    scopes: HashMap<u64, Scope>,
    membership: HashMap<u64, u64>,
}

/// A scope whose children have all finished, ready to release their retained results.
pub(in crate::vm) struct ClosedScope {
    pub(in crate::vm) tasks: Vec<u64>,
    pub(in crate::vm) failures: Vec<ScopeFailure>,
}

/// VM-wide registry of open task scopes and their children.
pub(in crate::vm) struct ScopeRegistry {
    state: Mutex<State>,
    next_id: AtomicU64,
    sequence: AtomicU64,
}

impl Default for ScopeRegistry {
    fn default() -> Self {
        Self {
            state: Mutex::default(),
            next_id: AtomicU64::new(1),
            sequence: AtomicU64::new(1),
        }
    }
}

impl ScopeRegistry {
    /// Open an empty scope and return its identity and shared control flags.
    pub(in crate::vm) fn open(&self) -> (u64, Arc<ScopeControl>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let control = Arc::new(ScopeControl::default());
        self.lock().scopes.insert(
            id,
            Scope {
                control: Arc::clone(&control),
                children: Vec::new(),
                failures: Vec::new(),
            },
        );
        (id, control)
    }

    /// Register a child before it is queued, so its completion is always observed.
    pub(in crate::vm) fn enroll(&self, scope: u64, task: u64) -> Option<Arc<ScopeControl>> {
        let mut state = self.lock();
        let entry = state.scopes.get_mut(&scope)?;
        entry.children.push(Child {
            id: task,
            done: false,
        });
        let control = Arc::clone(&entry.control);
        state.membership.insert(task, scope);
        Some(control)
    }

    /// Record a child's terminal outcome and report whether a scope owns it.
    ///
    /// A failure that is not a cancellation fails the scope and cancels its other children.
    pub(in crate::vm) fn complete(&self, task: u64, failure: Option<&VmError>) -> bool {
        let sequence = self.next_sequence();
        let mut state = self.lock();
        let Some(scope) = state.membership.get(&task).copied() else {
            return false;
        };
        let Some(entry) = state.scopes.get_mut(&scope) else {
            return true;
        };
        let Some(start_order) = entry.children.iter().position(|child| child.id == task) else {
            return true;
        };
        if entry.children[start_order].done {
            return true;
        }
        entry.children[start_order].done = true;
        if let Some(error) = failure.filter(|error| !is_cancellation(error)) {
            entry.failures.push(ScopeFailure {
                sequence,
                start_order,
                error: error.clone(),
            });
            entry.control.fail();
        }
        true
    }

    /// Request cancellation of every child of one scope.
    pub(in crate::vm) fn cancel(&self, scope: u64) {
        if let Some(entry) = self.lock().scopes.get(&scope) {
            entry.control.cancel();
        }
    }

    /// Observation identity of the failure recorded for an owned child.
    pub(in crate::vm) fn failure_sequence(&self, task: u64) -> Option<u64> {
        let state = self.lock();
        let scope = state.scopes.get(state.membership.get(&task)?)?;
        let start_order = scope.children.iter().position(|child| child.id == task)?;
        scope
            .failures
            .iter()
            .find(|failure| failure.start_order == start_order)
            .map(|failure| failure.sequence)
    }

    /// Release a scope once every child has finished.
    pub(in crate::vm) fn try_close(&self, scope: u64) -> Option<ClosedScope> {
        let mut state = self.lock();
        let Some(entry) = state.scopes.get(&scope) else {
            return Some(ClosedScope {
                tasks: Vec::new(),
                failures: Vec::new(),
            });
        };
        if entry.children.iter().any(|child| !child.done) {
            return None;
        }
        let entry = state.scopes.remove(&scope)?;
        let mut tasks = Vec::with_capacity(entry.children.len());
        for child in entry.children {
            state.membership.remove(&child.id);
            tasks.push(child.id);
        }
        Some(ClosedScope {
            tasks,
            failures: entry.failures,
        })
    }

    /// Cancel every open scope before runtime teardown.
    pub(in crate::vm) fn shutdown(&self) {
        for entry in self.lock().scopes.values() {
            entry.control.cancel();
        }
    }

    /// Return a monotonically increasing observation order for failures.
    pub(in crate::vm) fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::Relaxed)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}
