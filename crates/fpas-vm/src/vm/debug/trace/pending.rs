//! Commit-local ordering shared by global hooks and local/alias observations.

use super::model::TraceLeaf;
use super::{TraceHistory, local_history::MAX_LOCAL_INSTANCES};
use fpas_bytecode::Executable;

#[derive(Clone, Copy)]
pub(super) struct TraceSite<'a> {
    pub task: u64,
    pub function: Option<&'a str>,
    pub location: Option<(&'a str, u32, u32)>,
    pub commit: Option<u64>,
}

pub(super) struct PendingCommit {
    task: u64,
    function: Option<String>,
    location: Option<(String, u32, u32)>,
    changes: Vec<PendingChange>,
}

struct PendingChange {
    watch: usize,
    old: Option<TraceLeaf>,
    new: Option<TraceLeaf>,
    context: Option<(u64, u64)>,
}

impl TraceHistory {
    pub(super) fn commit_id(&mut self) -> u64 {
        match self.next_commit.checked_add(1) {
            Some(id) => {
                self.next_commit = id;
                id
            }
            None => {
                self.fail("trace_value_limit", false);
                self.next_commit
            }
        }
    }

    pub(super) fn queue(
        &mut self,
        id: u64,
        watch: usize,
        values: (Option<TraceLeaf>, Option<TraceLeaf>),
        site: TraceSite<'_>,
        context: Option<(u64, u64)>,
    ) {
        let (old, new) = values;
        if site.function.is_some_and(|name| name.len() > 4096)
            || site.location.is_some_and(|(name, _, _)| name.len() > 4096)
        {
            self.fail("trace_byte_limit", false);
            return;
        }
        let pending = self.pending.entry(id).or_insert_with(|| PendingCommit {
            task: site.task,
            function: site.function.map(str::to_owned),
            location: site
                .location
                .map(|(name, line, column)| (name.into(), line, column)),
            changes: Vec::new(),
        });
        if pending.changes.len() >= MAX_LOCAL_INSTANCES + 64 {
            self.fail("trace_value_limit", false);
            return;
        }
        pending.changes.push(PendingChange {
            watch,
            old,
            new,
            context,
        });
    }

    pub(super) fn flush(&mut self, id: u64, image: &Executable) {
        let Some(mut pending) = self.pending.remove(&id) else {
            return;
        };
        pending.changes.sort_by_key(|change| change.watch);
        let site = TraceSite {
            task: pending.task,
            function: pending.function.as_deref(),
            location: pending
                .location
                .as_ref()
                .map(|(name, line, column)| (name.as_str(), *line, *column)),
            commit: None,
        };
        for change in pending.changes {
            self.record_change(
                image,
                change.watch,
                change.old,
                change.new,
                site,
                change.context,
            );
        }
    }
}
