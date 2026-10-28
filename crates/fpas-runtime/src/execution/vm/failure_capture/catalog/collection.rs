//! One bounded cooperative cut; inspection never executes queued work.

use super::super::{
    CaptureState, FailureObservation, FailureTask, FailureTaskCatalog, identity::Context,
};
use super::selection::{CutTask, add_cause, add_causes, cause_ids, trim};
use super::*;
use crate::execution::vm::TaskResultState;
use std::{
    collections::HashMap,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

impl CaptureState {
    /// Collect once before report delivery; no runtime locks survive the call.
    pub(in crate::execution::vm::failure_capture) fn collect_tasks(
        self: &Arc<Self>,
        observation: &FailureObservation,
    ) -> Arc<FailureTaskCatalog> {
        let deadline = Instant::now() + Duration::from_millis(500);
        let (capacity, lease) = self.reserve_collection(
            MAX_TASKS,
            std::mem::size_of::<FailureTask>() + std::mem::size_of::<CutTask>() + 192,
        );
        // Root and primary identities survive even if optional model storage is exhausted.
        let (capacity, lease) = if capacity < 2 {
            drop(lease);
            (
                2,
                self.reserve_terminal(
                    2 * (std::mem::size_of::<FailureTask>() + std::mem::size_of::<CutTask>() + 192),
                ),
            )
        } else {
            (capacity, lease)
        };
        let scheduler = self.scheduler.get().and_then(std::sync::Weak::upgrade);
        let cut = |results: &HashMap<u64, TaskResultState>| {
            // A compact census survives selection limits without retaining released history.
            let _identity_lease =
                self.reserve_terminal((super::super::MAX_RELATED + 2) * std::mem::size_of::<u64>());
            let known_causes = cause_ids(observation);
            let entries = self
                .catalog
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let mut tasks = BTreeMap::new();
            add_cause(observation, &mut tasks);
            tasks.entry(0).or_insert(CutTask {
                record: None,
                state: FAILED,
                fault: None,
            });
            for error in &observation.related_errors {
                add_causes(error, &mut tasks, capacity);
            }
            let preferred: Vec<_> = tasks.keys().copied().collect();
            let mut total = results.len()
                + entries
                    .keys()
                    .filter(|id| !results.contains_key(id))
                    .count();
            total += known_causes
                .iter()
                .filter(|id| !results.contains_key(id) && !entries.contains_key(id))
                .count();
            for (&id, record) in entries.iter() {
                tasks.entry(id).or_insert_with(|| CutTask {
                    record: Some(Arc::clone(record)),
                    state: record.state.load(Ordering::Acquire),
                    fault: None,
                });
                trim(&mut tasks, &preferred, capacity);
            }
            for (&id, result) in results {
                tasks.entry(id).or_insert(CutTask {
                    record: None,
                    state: match result {
                        TaskResultState::Pending => RUNNABLE,
                        TaskResultState::Failed(error) => {
                            if matches!(
                                error.code,
                                fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                                    | fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                            ) {
                                CANCELLED
                            } else {
                                FAILED
                            }
                        }
                        _ => COMPLETED,
                    },
                    fault: None,
                });
                trim(&mut tasks, &preferred, capacity);
            }
            for (&id, task) in tasks.iter_mut() {
                if let Some(record) = entries.get(&id) {
                    task.record = Some(Arc::clone(record));
                    task.state = record.state.load(Ordering::Acquire);
                }
                if let Some(TaskResultState::Failed(error)) = results.get(&id)
                    && task.fault.is_none()
                    && task.state == FAILED
                {
                    task.fault = error
                        .attachment::<Context>()
                        .map(|context| context.cause.freeze())
                        .filter(|snapshot| snapshot.task_id == Some(id));
                }
            }
            (tasks, total)
        };
        let (selected, total) = scheduler.as_ref().map_or_else(
            || cut(&HashMap::new()),
            |scheduler| scheduler.with_capture_results(cut),
        );
        self.catalog.collecting.store(true, Ordering::Release);
        let reset = CollectionGuard {
            state: self,
            records: selected
                .values()
                .filter_map(|task| task.record.as_ref())
                .cloned()
                .collect(),
        };
        let mut tasks = Vec::with_capacity(selected.len());
        // Primary and root values have already been retained; other peers follow ascending IDs.
        for (id, task) in selected {
            let fault = task.fault.or_else(|| {
                task.record
                    .as_ref()
                    .and_then(|record| {
                        record
                            .failure
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .upgrade()
                    })
                    .map(|cause| cause.freeze())
            });
            let terminal = task.state >= COMPLETED;
            let snapshot = fault.as_ref().map(Arc::clone).or_else(|| {
                task.record.as_ref().and_then(|record| {
                    record
                        .response
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .as_ref()
                        .map(Arc::clone)
                })
            });
            if snapshot.is_none()
                && !terminal
                && let Some(record) = &task.record
            {
                record.requested.store(true, Ordering::Release);
            }
            let (name, name_reason, name_lease) = self.task_name(
                id,
                task.record
                    .as_ref()
                    .map(|record| record.function)
                    .or_else(|| fault.as_ref().and_then(|snapshot| snapshot.entry_function)),
                observation.cause.task_id == Some(id) || id == 0,
            );
            let absent = if terminal && task.state == FAILED && snapshot.is_none() {
                Some("no_live_frame")
            } else {
                None
            };
            tasks.push(FailureTask {
                id,
                name,
                state: state_name(task.state),
                observation: if fault.is_some() {
                    "fault"
                } else {
                    "failure_handling"
                },
                inspection: if snapshot.is_some() {
                    "available"
                } else if terminal && task.state != FAILED {
                    "not_applicable"
                } else {
                    "unavailable"
                },
                reason: absent.or(name_reason),
                snapshot,
                _lease: name_lease,
            });
        }
        loop {
            let observed = self.catalog.generation.load(Ordering::Acquire);
            let mut pending = false;
            for task in &mut tasks {
                if task.snapshot.is_some()
                    || task.inspection == "not_applicable"
                    || task.reason == Some("no_live_frame")
                {
                    continue;
                }
                let Some(record) = reset.records.iter().find(|record| record.id == task.id) else {
                    task.reason = Some("storage_unavailable");
                    continue;
                };
                if let Some(snapshot) = record
                    .response
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .take()
                    .or_else(|| self.inspect_saved(record, deadline))
                {
                    task.reason = snapshot.reason;
                    task.inspection = if snapshot.reason.is_some()
                        || snapshot.frames.iter().any(|frame| frame.reason.is_some())
                    {
                        "partial"
                    } else {
                        "available"
                    };
                    task.snapshot = Some(snapshot);
                    record.requested.store(false, Ordering::Release);
                } else if Instant::now() >= deadline {
                    task.reason = Some("collection_deadline");
                } else {
                    pending = true;
                }
            }
            if !pending || Instant::now() >= deadline {
                break;
            }
            let wait = deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(5));
            let signal = self
                .catalog
                .signal
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            // A change published during the scan is visible here; rescan instead of sleeping.
            if self.catalog.generation.load(Ordering::Acquire) == observed {
                let _ = self.catalog.changed.wait_timeout(signal, wait);
            }
        }
        for task in &mut tasks {
            if let Some(snapshot) = &task.snapshot {
                task.reason = super::super::task_model::snapshot_reason(snapshot).or(task.reason);
                task.inspection = if task.reason.is_some() {
                    "partial"
                } else {
                    "available"
                };
            }
            if task.snapshot.is_none()
                && task.inspection != "not_applicable"
                && task.reason.is_none()
            {
                task.reason = Some("collection_deadline");
            }
        }
        drop(reset);
        let omitted = total.saturating_sub(tasks.len());
        let globals = self
            .root_globals
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .map(Arc::clone);
        let globals_status = globals.as_ref().map_or("unavailable", |globals| {
            if globals.reason.is_some()
                || globals
                    .bindings
                    .iter()
                    .any(|binding| super::super::task_model::value_reason(&binding.value).is_some())
            {
                "partial"
            } else {
                "available"
            }
        });
        Arc::new(FailureTaskCatalog {
            tasks,
            total_tasks: total,
            omitted_tasks: omitted,
            globals,
            globals_status,
            reason: (omitted > 0).then_some(if capacity < MAX_TASKS {
                "retained_byte_limit"
            } else {
                "task_limit"
            }),
            _lease: lease,
        })
    }
}

struct CollectionGuard<'a> {
    state: &'a CaptureState,
    records: Vec<Arc<TaskRecord>>,
}
impl Drop for CollectionGuard<'_> {
    fn drop(&mut self) {
        self.state
            .catalog
            .collecting
            .store(false, Ordering::Release);
        for record in &self.records {
            record.requested.store(false, Ordering::Release);
            record
                .response
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .take();
        }
    }
}
