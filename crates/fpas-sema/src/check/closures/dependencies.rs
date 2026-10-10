//! Complete recursive capture metadata after the referenced routine is analyzed.
//! See `docs/pascal/language/functions/closures.md`.

use std::collections::{HashMap, VecDeque};

use super::{CaptureBinding, task_bound_from_captures};
use crate::check::Checker;

/// A routine whose captures depend on a named target still being checked.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureOwner {
    /// A named routine declaration.
    Routine(usize),
    /// An anonymous closure expression.
    Closure(usize),
}

#[derive(Clone, Copy)]
struct Consumer {
    owner: CaptureOwner,
    scope_index: usize,
}

/// Reverse dependencies and lexical declaration depths for recursive captures.
#[derive(Default)]
pub(crate) struct CaptureDependencies {
    /// Binding provenance and deferred task/discard boundaries.
    pub(super) capabilities: super::pending_capabilities::PendingCapabilities,
    /// Pending named targets referenced by each anonymous closure.
    pub(crate) closures: HashMap<usize, Vec<usize>>,
    consumers: HashMap<usize, Vec<Consumer>>,
    depths: HashMap<(String, u32, usize, usize), usize>,
}

impl Checker {
    /// Retains declaration depths before the routine's lexical scope is removed.
    pub(in crate::check) fn remember_capture_scopes(&mut self, captures: &[CaptureBinding]) {
        for capture in captures {
            if let Some(depth) = self
                .scopes
                .capture_scope_index(&capture.name, capture.declaration)
            {
                self.capture_dependencies
                    .depths
                    .insert(identity(capture), depth);
            }
        }
    }

    /// Registers consumers without treating their own parameters or locals as captures.
    pub(in crate::check) fn register_capture_dependencies(
        &mut self,
        owner: CaptureOwner,
        scope_index: usize,
        targets: &[usize],
    ) {
        if let CaptureOwner::Closure(key) = owner {
            self.capture_dependencies
                .closures
                .insert(key, targets.to_vec());
        }
        for &target in targets {
            self.capture_dependencies
                .consumers
                .entry(target)
                .or_default()
                .push(Consumer { owner, scope_index });
        }
    }

    /// Propagates new captures through recursive dependencies until no metadata changes.
    pub(in crate::check) fn propagate_pending_captures(&mut self, target: usize) {
        let mut pending = VecDeque::from([target]);
        while let Some(target) = pending.pop_front() {
            let Some(source) = self.nested_routine_captures.get(&target) else {
                continue;
            };
            let source = source.captures.clone();
            let consumers = self
                .capture_dependencies
                .consumers
                .get(&target)
                .cloned()
                .unwrap_or_default();
            for consumer in consumers {
                let captures = match consumer.owner {
                    CaptureOwner::Routine(key) => self
                        .nested_routine_captures
                        .get_mut(&key)
                        .map(|info| &mut info.captures),
                    CaptureOwner::Closure(key) => self
                        .closure_infos
                        .get_mut(&key)
                        .map(|info| &mut info.captures),
                };
                let Some(captures) = captures else {
                    continue;
                };
                if !merge_captures(
                    captures,
                    &source,
                    consumer.scope_index,
                    &self.capture_dependencies.depths,
                ) {
                    continue;
                }
                let task_bound = task_bound_from_captures(captures);
                let task_free = captures.iter().all(|capture| capture.task_free);
                match consumer.owner {
                    CaptureOwner::Routine(key) => {
                        if let Some(info) = self.nested_routine_captures.get_mut(&key) {
                            info.task_bound = task_bound;
                        }
                        self.scopes
                            .update_routine_capture_capabilities(key, task_bound, task_free);
                        pending.push_back(key);
                    }
                    CaptureOwner::Closure(key) => {
                        if let Some(info) = self.closure_infos.get_mut(&key) {
                            info.task_bound = task_bound;
                        }
                        if task_bound {
                            self.mark_expr_task_bound(key);
                        }
                        if let Some(info) = self.discard_exprs.get_mut(&key) {
                            info.value = task_free;
                        }
                    }
                }
            }
        }
    }
}

fn identity(capture: &CaptureBinding) -> (String, u32, usize, usize) {
    (
        capture.name.to_ascii_lowercase(),
        capture.declaration.source_id,
        capture.declaration.offset,
        capture.declaration.length,
    )
}

fn merge_captures(
    captures: &mut Vec<CaptureBinding>,
    source: &[CaptureBinding],
    scope_index: usize,
    depths: &HashMap<(String, u32, usize, usize), usize>,
) -> bool {
    let mut changed = false;
    for capture in source {
        let key = identity(capture);
        if !depths
            .get(&key)
            .is_some_and(|&depth| depth > 0 && depth < scope_index)
        {
            continue;
        }
        if let Some(existing) = captures
            .iter_mut()
            .find(|existing| identity(existing) == key)
        {
            let task_bound = existing.task_bound || capture.task_bound;
            let task_free = existing.task_free && capture.task_free;
            changed |= existing.task_bound != task_bound || existing.task_free != task_free;
            existing.task_bound = task_bound;
            existing.task_free = task_free;
        } else {
            captures.push(capture.clone());
            changed = true;
        }
    }
    if changed {
        captures.sort_by_key(|capture| depths.get(&identity(capture)).copied());
    }
    changed
}
