//! Single-use selection cases owned by their creating VM and task.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_bytecode::{SharedFunction, Value};

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
}

struct Entry {
    owner: u64,
    case: WaitCase,
}

/// Unused cases; claiming transfers ownership to the selecting operation.
#[derive(Default)]
pub(in crate::vm) struct CaseRegistry {
    entries: Mutex<HashMap<u64, Entry>>,
}

impl CaseRegistry {
    /// Check capacity before retaining a case's values and callback.
    pub(super) fn create(
        &self,
        owner: u64,
        build: impl FnOnce() -> WaitCase,
    ) -> Result<u64, String> {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries.len() >= MAX_UNUSED {
            return Err("At most 4096 unused WaitCase handles may belong to one VM; select or close existing cases".into());
        }
        let handle = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                (id & MASK == TAG && id < (TAG | !MASK)).then(|| id + 1)
            })
            .map_err(|_| "WaitCase identity space is exhausted".to_string())?;
        entries.insert(
            handle,
            Entry {
                owner,
                case: build(),
            },
        );
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
            let entry = entries.get(id).ok_or_else(|| {
                "WaitCase is closed, already selected, or belongs to another VM".to_string()
            })?;
            if entry.owner != owner {
                return Err("WaitCase must be selected by the task that created it".into());
            }
        }
        Ok(ids
            .iter()
            .filter_map(|id| entries.remove(id).map(|entry| entry.case))
            .collect())
    }

    /// Discard an unused case, retaining no tombstone or captured references.
    pub(super) fn close(&self, owner: u64, id: u64) -> Result<bool, String> {
        validate(id)?;
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries.get(&id).is_some_and(|entry| entry.owner != owner) {
            return Err("WaitCase must be closed by the task that created it".into());
        }
        Ok(entries.remove(&id).is_some())
    }
}

fn validate(id: u64) -> Result<(), String> {
    if id & MASK != TAG {
        return Err("Expected a WaitCase handle created by Std.Task".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
