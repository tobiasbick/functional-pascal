//! Single-use selection cases owned by their creating VM and task.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{MemoryDomain, RootKind, RootValues, SharedFunction, TraceRoots, Value};

const TAG: u64 = 0x5345_0000_0000_0000;
const MASK: u64 = 0xffff_0000_0000_0000;
const MAX_UNUSED: usize = 4096;
pub(super) const MAX_CASES: usize = 1024;
static NEXT: AtomicU64 = AtomicU64::new(TAG | 1);

/// The source operation committed only when this case wins.
pub(super) enum CaseSource {
    Receive(u64),
    Send(u64, Value),
    Task(u64),
    Timer(u64),
    Cancellation(u64),
}

/// A source and its statically checked delivery callback.
pub(super) struct WaitCase {
    pub(super) source: CaseSource,
    pub(super) callback: SharedFunction,
    pub(super) roots: Option<RootValues>,
}

impl TraceRoots for WaitCase {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), crate::OwnershipError> {
        visit(&Value::Function(self.callback.clone()));
        if let CaseSource::Send(_, value) = &self.source {
            visit(value);
        }
        Ok(())
    }
}

struct Entry {
    owner: u64,
    case: WaitCase,
}

#[derive(Default)]
struct Entries {
    cases: HashMap<u64, Entry>,
    closed: bool,
}

/// Unused cases; claiming transfers ownership to the selecting operation.
#[derive(Default)]
pub(in crate::execution::vm) struct CaseRegistry {
    entries: Mutex<Entries>,
}

impl CaseRegistry {
    /// Check capacity before retaining a case's values and callback.
    #[cfg(test)]
    pub(super) fn create(
        &self,
        owner: u64,
        build: impl FnOnce() -> WaitCase,
    ) -> Result<u64, String> {
        self.create_in_domain(owner, &MemoryDomain::new(), build)
    }

    pub(super) fn create_in_domain(
        &self,
        owner: u64,
        domain: &MemoryDomain,
        build: impl FnOnce() -> WaitCase,
    ) -> Result<u64, String> {
        {
            let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
            entries.require_capacity()?;
        }
        let mut case = build();
        if let CaseSource::Send(_, payload) = &case.source {
            crate::execution::vm::domain::check_transfer(std::slice::from_ref(payload))?;
        }
        let mut values = Vec::new();
        case.trace_roots(&mut |value| values.push(value.clone()))
            .map_err(|e| e.to_string())?;
        case.roots = Some(domain.root_values(RootKind::Selection, values));
        // Construction and graph validation hold no registry lock. Check again
        // so concurrent shutdown or another constructor cannot admit a late case.
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.require_capacity()?;
        let handle = NEXT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                (id & MASK == TAG && id < (TAG | !MASK)).then(|| id + 1)
            })
            .map_err(|_| "WaitCase identity space is exhausted".to_string())?;
        entries.cases.insert(handle, Entry { owner, case });
        Ok(handle)
    }

    /// Validate all identities before atomically removing the complete selection input.
    pub(super) fn claim(&self, owner: u64, ids: &[u64]) -> Result<Vec<WaitCase>, String> {
        if !(1..=MAX_CASES).contains(&ids.len()) {
            return Err("Select requires between 1 and 1024 WaitCase handles".into());
        }
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let mut unique = HashSet::with_capacity(ids.len());
        for id in ids {
            validate(*id)?;
            if !unique.insert(*id) {
                return Err("Select cannot contain the same WaitCase handle twice".into());
            }
            let entry = entries.cases.get(id).ok_or_else(|| {
                "WaitCase is closed, already selected, or belongs to another VM".to_string()
            })?;
            if entry.owner != owner {
                return Err("WaitCase must be selected by the task that created it".into());
            }
        }
        Ok(ids
            .iter()
            .filter_map(|id| entries.cases.remove(id).map(|entry| entry.case))
            .collect())
    }

    /// Discard an unused case, retaining no tombstone or captured references.
    pub(super) fn close(&self, owner: u64, id: u64) -> Result<bool, String> {
        validate(id)?;
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries
            .cases
            .get(&id)
            .is_some_and(|entry| entry.owner != owner)
        {
            return Err("WaitCase must be closed by the task that created it".into());
        }
        let removed = entries.cases.remove(&id);
        drop(entries);
        Ok(removed.is_some())
    }

    /// Release unused cases after owner cleanup, before completion or a retry.
    pub(in crate::execution::vm) fn close_owner(&self, owner: u64) -> usize {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let removed = entries
            .cases
            .extract_if(|_, entry| entry.owner == owner)
            .map(|(_, entry)| entry)
            .collect::<Vec<_>>();
        drop(entries);
        let count = removed.len();
        drop(removed);
        count
    }

    /// Shut admission and release unused cases without invoking their callbacks.
    pub(in crate::execution::vm) fn close_all(&self) -> usize {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.closed = true;
        let removed = std::mem::take(&mut entries.cases);
        drop(entries);
        let count = removed.len();
        drop(removed);
        count
    }
}

impl Entries {
    fn require_capacity(&self) -> Result<(), String> {
        if self.closed {
            return Err("WaitCase registry is closed after session shutdown".into());
        }
        if self.cases.len() >= MAX_UNUSED {
            return Err("At most 4096 unused WaitCase handles may belong to one VM; select or close existing cases".into());
        }
        Ok(())
    }
}

fn validate(id: u64) -> Result<(), String> {
    if id & MASK != TAG {
        return Err("Expected a WaitCase handle created by Std.Tasks".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
