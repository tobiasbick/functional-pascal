//! Atomic session ownership for global data breakpoints.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::*;
use crate::vm::debug::breakpoints::{
    BoundDataBreakpoint, DataBreakpoint, DataBreakpointAccess, bind_data,
};
use crate::vm::debug::location::DebugDataLocationIdentity;
use fpas_bytecode::Value;

use super::breakpoints::breakpoint_id_limit;

impl DebugSession {
    /// Atomically replace every data breakpoint in the session.
    ///
    /// Only executable-global write and change watches are verified. Read
    /// watches and live-frame identities remain unverified without mutation of
    /// previously accepted data breakpoints when the request is rejected.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or breakpoint-limit error without changing the
    /// existing data breakpoints or the next logical identifier.
    pub fn replace_data_breakpoints(
        &mut self,
        requested: Vec<DataBreakpoint>,
    ) -> Result<Vec<BoundDataBreakpoint>, DebugSessionError> {
        self.require_stopped("data_breakpoints.replace")?;
        self.require_breakpoint_capacity(
            self.source_breakpoints.len() + self.function_breakpoints.len() + requested.len(),
        )?;
        let count = u64::try_from(requested.len()).map_err(|_| breakpoint_id_limit())?;
        let next_id = self
            .next_breakpoint_id
            .checked_add(count)
            .ok_or_else(breakpoint_id_limit)?;
        let bound = requested
            .into_iter()
            .enumerate()
            .map(|(offset, request)| {
                let offset = u64::try_from(offset).unwrap_or(u64::MAX);
                bind_data(self.next_breakpoint_id + offset, request)
            })
            .collect::<Vec<_>>();
        self.data_breakpoints.clone_from(&bound);
        self.next_breakpoint_id = next_id;
        self.refresh_data_watch_snapshots();
        Ok(bound)
    }

    pub(super) fn take_data_breakpoint_hits(&mut self, task_id: u64) -> Vec<u64> {
        let stored = self
            .runtime
            .worker_mut(task_id)
            .and_then(Worker::take_debug_global_store);
        if self.data_breakpoints.is_empty() {
            return Vec::new();
        }
        let Some(stored) = stored else {
            return Vec::new();
        };
        let mut hits = Vec::new();
        for breakpoint in &self.data_breakpoints {
            if !breakpoint.verified {
                continue;
            }
            let DebugDataLocationIdentity::Global { index } = breakpoint.requested.identity else {
                continue;
            };
            let stored_hit = stored == u32::try_from(index).unwrap_or(u32::MAX);
            let changed = stored_hit
                && self.global_value(index)
                    != self.data_watch_snapshots.get(&index).cloned().flatten();
            let hit = match breakpoint.requested.access {
                DataBreakpointAccess::Write => stored_hit,
                DataBreakpointAccess::Change => changed,
                DataBreakpointAccess::Read => false,
            };
            if hit {
                hits.push(breakpoint.id);
            }
        }
        hits.sort_unstable();
        hits
    }

    pub(super) fn refresh_data_watch_snapshots(&mut self) {
        let mut snapshots = BTreeMap::new();
        for breakpoint in &self.data_breakpoints {
            if !breakpoint.verified {
                continue;
            }
            if let DebugDataLocationIdentity::Global { index } = breakpoint.requested.identity {
                snapshots.insert(index, self.global_value(index));
            }
        }
        self.data_watch_snapshots = snapshots;
    }

    fn global_value(&self, index: u64) -> Option<Value> {
        let index = usize::try_from(index).ok()?;
        let worker = self.runtime.worker(0)?;
        let globals = worker
            .globals
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        globals.get(index).cloned().flatten()
    }
}
