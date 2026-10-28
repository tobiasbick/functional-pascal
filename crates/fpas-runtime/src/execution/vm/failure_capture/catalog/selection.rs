//! Bounded task selection and a complete census of known structured cause identities.

use super::*;
use crate::execution::vm::failure_capture::FailureObservation;

/// Selected metadata at the catalog cut, before any peer request.
pub(super) struct CutTask {
    /// Stable lifecycle owner, if admitted within the catalog bounds.
    pub(super) record: Option<Arc<TaskRecord>>,
    /// State observed at selection time.
    pub(super) state: u8,
    /// Original cause frames take priority over later peer responses.
    pub(super) fault: Option<Arc<FailureSnapshot>>,
}

/// Known cause IDs include released tasks, even when their catalog entries are omitted.
pub(super) fn cause_ids(observation: &FailureObservation) -> Vec<u64> {
    let mut ids = Vec::with_capacity(super::super::MAX_RELATED + 2);
    ids.push(0);
    collect_ids(observation, &mut ids);
    ids
}

fn collect_ids(observation: &FailureObservation, ids: &mut Vec<u64>) {
    if let Some(id) = observation.cause.task_id
        && !ids.contains(&id)
    {
        ids.push(id);
    }
    for error in &observation.related_errors {
        collect_ids(error, ids);
    }
}

/// Add preferred cause frames without exceeding the bounded selection.
pub(super) fn add_causes(
    observation: &FailureObservation,
    tasks: &mut BTreeMap<u64, CutTask>,
    maximum: usize,
) {
    if tasks.len() < maximum
        || observation
            .cause
            .task_id
            .is_some_and(|id| tasks.contains_key(&id))
    {
        add_cause(observation, tasks);
    }
    for error in &observation.related_errors {
        add_causes(error, tasks, maximum);
    }
}

/// Fill existing placeholders while preserving an already selected primary fault.
pub(super) fn add_cause(observation: &FailureObservation, tasks: &mut BTreeMap<u64, CutTask>) {
    if let Some(id) = observation.cause.task_id {
        let state = if matches!(
            observation.error.code,
            fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                | fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
        ) {
            CANCELLED
        } else {
            FAILED
        };
        let task = tasks.entry(id).or_insert(CutTask {
            record: None,
            state,
            fault: None,
        });
        if task.fault.is_none() {
            task.state = state;
            task.fault = Some(Arc::clone(&observation.cause));
        }
    }
}

/// Keep preferred identities, then the smallest remaining runtime task IDs.
pub(super) fn trim(tasks: &mut BTreeMap<u64, CutTask>, preferred: &[u64], maximum: usize) {
    if tasks.len() > maximum
        && let Some(id) = tasks
            .keys()
            .rev()
            .find(|id| !preferred.contains(id))
            .copied()
    {
        tasks.remove(&id);
    }
}
