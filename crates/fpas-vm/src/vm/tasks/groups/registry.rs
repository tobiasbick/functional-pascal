//! Bounded group membership and retained child completion reports.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::vm::VmError;
use crate::vm::cancellation::OwnedCancellation;
use fpas_bytecode::Value;
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_TASK_CANCELLED, RUNTIME_VM_SHUTDOWN};

const TAG: u64 = 0x4752_0000_0000_0000;
const MASK: u64 = 0xffff_0000_0000_0000;
static NEXT: AtomicU64 = AtomicU64::new(TAG | 1);

/// Backing values of the payload-free `Std.Task.TaskFailureKind` enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub(in crate::vm) enum GroupFailureKind {
    /// A worker returned a top-level Result error.
    ReturnedError = 0,
    /// A worker executed a Pascal panic.
    Panicked = 1,
    /// A worker failed with another runtime diagnostic.
    RuntimeError = 2,
    /// The runtime cancelled a worker.
    Cancelled = 3,
}

/// Compact failure report retained independently of a child's consumable task result.
#[derive(Clone)]
pub(in crate::vm) struct GroupFailure {
    pub(in crate::vm) task: u64,
    pub(in crate::vm) kind: GroupFailureKind,
    pub(in crate::vm) message: String,
    pub(in crate::vm) code: u16,
    pub(in crate::vm) line: u32,
    pub(in crate::vm) column: u32,
}

impl GroupFailure {
    /// Classify an ordinary returned error without guessing cancellation from its message.
    pub(in crate::vm) fn returned(task: u64, value: &Value) -> Option<Self> {
        let Value::ResultError(payload) = value else {
            return None;
        };
        let message = match payload.as_ref() {
            Value::Str(message) => message.chars().take(4096).collect(),
            _ => "Task returned an invalid non-string error payload".into(),
        };
        Some(Self {
            task,
            kind: GroupFailureKind::ReturnedError,
            message,
            code: 0,
            line: 0,
            column: 0,
        })
    }

    /// Preserve the class and location of a child runtime failure in a bounded report.
    pub(in crate::vm) fn runtime(task: u64, error: &VmError) -> Self {
        let kind = if error.code == RUNTIME_PROGRAM_PANIC {
            GroupFailureKind::Panicked
        } else if matches!(error.code, RUNTIME_TASK_CANCELLED | RUNTIME_VM_SHUTDOWN) {
            GroupFailureKind::Cancelled
        } else {
            GroupFailureKind::RuntimeError
        };
        Self {
            task,
            kind,
            message: error.message.chars().take(4096).collect(),
            code: error.code.value(),
            line: error.span.line(),
            column: error.span.column(),
        }
    }
}

struct Child {
    id: u64,
    outcome: Option<Option<GroupFailure>>,
}

struct Group {
    owner: u64,
    cancellation: OwnedCancellation,
    closing: bool,
    children: Vec<Child>,
}

#[derive(Default)]
struct State {
    groups: HashMap<u64, Group>,
    membership: HashMap<u64, u64>,
}

/// Finished ownership transferred to the scheduler for result release.
pub(in crate::vm) struct ClosedGroup {
    pub(in crate::vm) tasks: Vec<u64>,
    pub(in crate::vm) failures: Vec<GroupFailure>,
}

/// VM-local group state shared across normal and debugger scheduling.
#[derive(Default)]
pub(in crate::vm) struct GroupRegistry {
    state: Mutex<State>,
}

impl GroupRegistry {
    /// Report whether ownership and its child failure reports still require explicit close.
    pub(in crate::vm) fn is_live(&self, id: u64) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .groups
            .contains_key(&id)
    }
    /// Create a group only after checking the live-resource bound.
    pub(in crate::vm) fn create(
        &self,
        owner: u64,
        source: impl FnOnce() -> OwnedCancellation,
    ) -> Result<u64, String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.groups.len() >= 4096 {
            return Err("At most 4096 task groups may be live; close existing groups".into());
        }
        let id = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                (id < (TAG | !MASK)).then(|| id + 1)
            })
            .map_err(|_| "TaskGroup identity space exhausted".to_string())?;
        state.groups.insert(
            id,
            Group {
                owner,
                cancellation: source(),
                closing: false,
                children: vec![],
            },
        );
        Ok(id)
    }

    /// Return the group-owned cancellation token.
    pub(in crate::vm) fn token(&self, id: u64) -> Result<u64, String> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        Ok(group(&state, id)?.cancellation.token())
    }

    /// Atomically retain a child before it can be queued or its handle published.
    pub(in crate::vm) fn enroll(&self, id: u64, caller: u64, task: u64) -> Result<u64, String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let member = state.membership.get(&caller) == Some(&id);
        let group = state.groups.get_mut(&id).ok_or_else(unknown)?;
        if group.owner != caller && !member {
            return Err("Only a task group's owner or children may start its tasks".into());
        }
        if group.closing || group.cancellation.is_cancelled() {
            return Err("Cannot start a task in a closing or cancelled group".into());
        }
        if group.children.len() >= 1024 {
            return Err("A task group may register at most 1024 children".into());
        }
        let token = group.cancellation.token();
        group.children.push(Child {
            id: task,
            outcome: None,
        });
        state.membership.insert(task, id);
        Ok(token)
    }

    /// Publish a child's first terminal outcome and report whether it belongs to a group.
    pub(in crate::vm) fn complete(&self, task: u64, failure: Option<GroupFailure>) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let Some(id) = state.membership.get(&task).copied() else {
            return false;
        };
        if let Some(child) = state
            .groups
            .get_mut(&id)
            .and_then(|group| group.children.iter_mut().find(|child| child.id == task))
            && child.outcome.is_none()
        {
            child.outcome = Some(failure);
        }
        true
    }

    /// Request cancellation without releasing task ownership.
    pub(in crate::vm) fn cancel(&self, id: u64) -> Result<bool, String> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        Ok(group(&state, id)?.cancellation.cancel())
    }

    /// Seal admission and request cooperative cancellation before joining children.
    pub(in crate::vm) fn begin_close(&self, id: u64, owner: u64) -> Result<(), String> {
        validate(id)?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let Some(group) = state.groups.get_mut(&id) else {
            return Ok(());
        };
        if group.owner != owner {
            return Err("Only the creating task may close a TaskGroup".into());
        }
        group.closing = true;
        group.cancellation.cancel();
        Ok(())
    }

    /// Release a closed group only after every registered child has a terminal outcome.
    pub(in crate::vm) fn take_closed(&self, id: u64) -> Result<Option<ClosedGroup>, String> {
        validate(id)?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let Some(group) = state.groups.get(&id) else {
            return Ok(Some(ClosedGroup {
                tasks: vec![],
                failures: vec![],
            }));
        };
        if !group.closing {
            return Err("CloseTaskGroup must seal admission before joining".into());
        }
        if group.children.iter().any(|child| child.outcome.is_none()) {
            return Ok(None);
        }
        Ok(Some(take_complete(&mut state, id)?))
    }

    /// Seal and release a group only when every child has already reached a terminal outcome.
    pub(in crate::vm) fn try_take_completed(
        &self,
        id: u64,
        owner: u64,
    ) -> Result<Option<ClosedGroup>, String> {
        validate(id)?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let Some(group) = state.groups.get(&id) else {
            return Ok(Some(ClosedGroup {
                tasks: vec![],
                failures: vec![],
            }));
        };
        if group.owner != owner {
            return Err("Only the creating task may close a TaskGroup".into());
        }
        if group.children.iter().any(|child| child.outcome.is_none()) {
            return Ok(None);
        }
        Ok(Some(take_complete(&mut state, id)?))
    }

    /// Request cancellation for all live groups before VM thread joining.
    pub(in crate::vm) fn shutdown(&self) {
        for group in self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .groups
            .values_mut()
        {
            group.closing = true;
            group.cancellation.cancel();
        }
    }
}

fn take_complete(state: &mut State, id: u64) -> Result<ClosedGroup, String> {
    let Some(group) = state.groups.remove(&id) else {
        return Err(unknown());
    };
    let mut closed = ClosedGroup {
        tasks: Vec::with_capacity(group.children.len()),
        failures: vec![],
    };
    for child in group.children {
        state.membership.remove(&child.id);
        closed.tasks.push(child.id);
        if let Some(Some(failure)) = child.outcome {
            closed.failures.push(failure);
        }
    }
    Ok(closed)
}

fn group(state: &State, id: u64) -> Result<&Group, String> {
    validate(id)?;
    state.groups.get(&id).ok_or_else(unknown)
}
fn unknown() -> String {
    "TaskGroup is closed or does not belong to this VM".into()
}
fn validate(id: u64) -> Result<(), String> {
    if id & MASK == TAG {
        Ok(())
    } else {
        Err("Expected a TaskGroup handle".into())
    }
}

#[cfg(test)]
mod tests;
