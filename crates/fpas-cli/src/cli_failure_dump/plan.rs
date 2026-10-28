//! Prioritize borrowed report components before encoding, preserving valid JSON at the bound.

use super::{
    budget::{self, Budget},
    sources::Sources,
    wire::*,
};
use fpas_diagnostics::Diagnostic;
use fpas_runtime::execution::{FailureObservation, FailureTask, FailureValue};
use std::{collections::BTreeSet, io};

/// Reserve causal identities, then primary values, globals, main and other peers.
pub(super) fn observation(data: &FailureObservation, sources: &Sources) -> io::Result<Vec<u8>> {
    let mut report = base(&data.error, data.cause.task_id);
    report.error.location = report
        .error
        .location
        .filter(|location| sources.contains(location.source_id));
    let mut reasons = sources.reasons.clone();
    observation_reasons(data, &mut reasons);
    let mut budget = Budget::new(&report);
    add_source(
        report.error.location,
        &mut report,
        sources,
        &mut budget,
        &mut reasons,
    );
    if let Some(catalog) = &data.task_catalog {
        report.capture.total_tasks = Some(catalog.total_tasks);
        report.capture.globals_status = catalog.globals_status;
        if let Some(reason) = catalog.reason {
            reasons.insert(reason);
        }
        if catalog.globals_status == "unavailable" {
            reasons.insert("storage_unavailable");
        }
        if let Some(globals) = &catalog.globals {
            if let Some(reason) = globals.reason {
                reasons.insert(reason);
            }
            for global in &globals.bindings {
                value_reasons(&global.value, &mut reasons);
            }
        }
        let mut causes = BTreeSet::new();
        cause_ids(data, &mut causes);
        let mut priority: Vec<_> = catalog.tasks.iter().collect();
        priority.sort_by_key(|task| {
            (
                if Some(task.id) == data.cause.task_id {
                    0
                } else if task.id == 0 {
                    1
                } else if causes.contains(&task.id) {
                    2
                } else {
                    3
                },
                task.id,
            )
        });
        // Values must not crowd out known causal identities or their short diagnostics.
        for task in &priority {
            task_reasons(task, &mut reasons);
            if (causes.contains(&task.id) || task.id == 0)
                && !add_task(task, &mut report, &mut budget)
            {
                reasons.insert("byte_limit");
            }
        }
        related(data, &mut report, sources, &mut budget, &mut reasons);
        if let Some(task) = priority
            .iter()
            .find(|task| Some(task.id) == data.cause.task_id)
        {
            frames(task, &mut report, sources, &mut budget, &mut reasons);
        }
        if let Some(globals) = &catalog.globals {
            let mut count = 0;
            let mut partial = None;
            for global in &globals.bindings {
                if !budget.take(&Binding::global(global), 0) {
                    reasons.insert("byte_limit");
                    report.capture.globals_status = "partial";
                    partial = budget.partial(|policy| Binding::global(global).with_policy(policy));
                    if partial.is_some() {
                        count += 1;
                    }
                    break;
                }
                count += 1;
            }
            report.globals = Globals(&globals.bindings[..count], partial);
        }
        for task in priority {
            if Some(task.id) == data.cause.task_id {
                continue;
            }
            if !report.tasks.iter().any(|entry| entry.id == task.id)
                && !add_task(task, &mut report, &mut budget)
            {
                reasons.insert("byte_limit");
                continue;
            }
            frames(task, &mut report, sources, &mut budget, &mut reasons);
        }
        report.capture.omitted_tasks = Some(catalog.total_tasks.saturating_sub(report.tasks.len()));
    } else {
        reasons.insert("storage_unavailable");
        related(data, &mut report, sources, &mut budget, &mut reasons);
    }
    report.tasks.sort_by_key(|task| task.id);
    report.sources.sort_by_key(|source| source.id);
    report.capture.incomplete = !reasons.is_empty();
    report.capture.reasons = reasons.into_iter().collect();
    budget::encode(&report)
        .or_else(|_| minimal(&data.error, data.cause.task_id, sources, data.error.reason))
}

/// Honest fallback when capture or full encoding could not provide a usable model.
pub(super) fn minimal(
    error: &Diagnostic,
    task_id: Option<u64>,
    sources: &Sources,
    text_omission: Option<&'static str>,
) -> io::Result<Vec<u8>> {
    let mut report = base(error, task_id);
    report.error.location = report
        .error
        .location
        .filter(|location| sources.contains(location.source_id));
    report.capture.incomplete = true;
    let mut reasons = sources.reasons.clone();
    reasons.insert("storage_unavailable");
    if let Some(reason) = text_omission {
        reasons.insert(reason);
    }
    if text_omission.is_some()
        || report.error.message.len() < error.message.len()
        || error
            .help
            .as_deref()
            .is_some_and(|hint| prefix(hint).len() < hint.len())
    {
        reasons.insert("byte_limit");
    }
    report.capture.reasons = reasons.into_iter().collect();
    if let Some(location) = report.error.location {
        report.sources.push(Source {
            id: location.source_id,
            path: sources.get(location.source_id),
        });
    }
    if let Some(id) = task_id {
        report.tasks.push(Task {
            id,
            name: None,
            state: "failed",
            observation: "fault",
            inspection: "unavailable",
            unavailable_reason: Some("storage_unavailable"),
            total_frames: None,
            frames: Vec::new(),
        });
    }
    budget::encode(&report)
}

fn base(error: &Diagnostic, task_id: Option<u64>) -> Report<'_> {
    Report {
        schema_version: 1,
        error: Error::new(error, task_id),
        related_errors: Vec::new(),
        sources: Vec::new(),
        capture: Capture {
            consistency: "cooperative",
            incomplete: false,
            reasons: Vec::new(),
            total_tasks: None,
            omitted_tasks: None,
            globals_status: "unavailable",
            limits: Limits::default(),
        },
        globals: Globals(&[], None),
        tasks: Vec::new(),
    }
}

fn add_task<'a>(task: &'a FailureTask, report: &mut Report<'a>, budget: &mut Budget) -> bool {
    let task = Task::new(task);
    if !budget.take(&task, 64) {
        return false;
    }
    report.tasks.push(task);
    true
}

fn frames<'a>(
    original: &'a FailureTask,
    report: &mut Report<'a>,
    sources: &'a Sources,
    budget: &mut Budget,
    reasons: &mut BTreeSet<&'static str>,
) {
    let Some(index) = report.tasks.iter().position(|task| task.id == original.id) else {
        return;
    };
    let Some(snapshot) = &original.snapshot else {
        return;
    };
    for original_frame in &snapshot.frames {
        let mut frame = Frame::new(original_frame);
        if !budget.take(&frame, 0) {
            report.tasks[index].limited();
            reasons.insert("byte_limit");
            break;
        }
        add_source(frame.location, report, sources, budget, reasons);
        let mut count = 0;
        let mut partial = None;
        for binding in &original_frame.bindings {
            if !budget.take(&Binding::local(binding), 0) {
                frame.truncated = true;
                report.tasks[index].limited();
                reasons.insert("byte_limit");
                partial = budget.partial(|policy| Binding::local(binding).with_policy(policy));
                if partial.is_some() {
                    count += 1;
                }
                break;
            }
            count += 1;
        }
        frame.bindings = Bindings(&original_frame.bindings[..count], partial);
        report.tasks[index].frames.push(frame);
    }
}

fn add_source<'a>(
    location: Option<Location>,
    report: &mut Report<'a>,
    sources: &'a Sources,
    budget: &mut Budget,
    reasons: &mut BTreeSet<&'static str>,
) {
    let Some(location) = location else {
        return;
    };
    if report
        .sources
        .iter()
        .any(|source| source.id == location.source_id)
    {
        return;
    }
    let source = Source {
        id: location.source_id,
        path: sources.get(location.source_id),
    };
    if source.path.is_none() {
        reasons.insert("missing_metadata");
    }
    if budget.take(&source, 0) {
        report.sources.push(source);
    } else {
        reasons.insert("byte_limit");
    }
}

fn related<'a>(
    data: &'a FailureObservation,
    report: &mut Report<'a>,
    sources: &'a Sources,
    budget: &mut Budget,
    reasons: &mut BTreeSet<&'static str>,
) {
    for related_error in &data.related_errors {
        let mut error = Error::new(&related_error.error, related_error.cause.task_id);
        error.location = error
            .location
            .filter(|location| sources.contains(location.source_id));
        if budget.take(&error, 0) {
            add_source(error.location, report, sources, budget, reasons);
            report.related_errors.push(error);
        } else {
            reasons.insert("byte_limit");
        }
        related(related_error, report, sources, budget, reasons);
    }
}

fn cause_ids(data: &FailureObservation, ids: &mut BTreeSet<u64>) {
    if let Some(id) = data.cause.task_id {
        ids.insert(id);
    }
    for error in &data.related_errors {
        cause_ids(error, ids);
    }
}

fn observation_reasons(data: &FailureObservation, reasons: &mut BTreeSet<&'static str>) {
    if let Some(reason) = data.error.reason {
        reasons.insert(reason);
        reasons.insert("byte_limit");
    }
    if let Some(reason) = data.cause.reason {
        reasons.insert(reason);
    }
    if data.related_errors_omitted > 0 {
        reasons.insert("retained_byte_limit");
    }
    for frame in &data.cause.frames {
        if let Some(reason) = frame.reason {
            reasons.insert(reason);
        }
        for binding in &frame.bindings {
            value_reasons(&binding.value, reasons);
        }
    }
    for error in &data.related_errors {
        observation_reasons(error, reasons);
    }
}

fn task_reasons(task: &FailureTask, reasons: &mut BTreeSet<&'static str>) {
    if let Some(reason) = task.reason {
        reasons.insert(reason);
    }
    if let Some(snapshot) = &task.snapshot {
        if let Some(reason) = snapshot.reason {
            reasons.insert(reason);
        }
        for frame in &snapshot.frames {
            if let Some(reason) = frame.reason {
                reasons.insert(reason);
            }
            for binding in &frame.bindings {
                value_reasons(&binding.value, reasons);
            }
        }
    }
}

fn value_reasons(value: &FailureValue, reasons: &mut BTreeSet<&'static str>) {
    if let Some(reason) = value.reason {
        reasons.insert(reason);
    }
    for (_, child) in &value.children {
        value_reasons(child, reasons);
    }
}
