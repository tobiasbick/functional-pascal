//! Detached, non-executing state for a one-shot debugger host.

use super::*;
use crate::vm::{
    failure_capture::{FailureFrame, FailureGlobal, FailureValue},
    inspection::{
        bindings::ReadValue,
        detached_frame,
        frames::{self, Boundary},
        globals,
        values::{ValueLimits, budget::ValueBudget, render::RetainedValue, snapshot},
    },
};
use crate::{DebugEvaluationLimits, DebugExpression};
use std::collections::{BTreeSet, HashSet};

const LIMITS: ValueLimits = ValueLimits {
    max_children: 64,
    max_string_chars: 4096,
    max_summary_chars: 4096,
    max_depth: 8,
};

/// Detached state for a real task, including explicit frame unavailability.
#[derive(Debug)]
pub struct DebugTaskSnapshot {
    /// Task identity and state from the all-stop catalog.
    pub task: super::super::types::DebugTask,
    /// Captured frames, active frame first.
    pub frames: Vec<FailureFrame>,
    /// Complete known frame count.
    pub total_frames: Option<usize>,
    /// Reason for omitted or unavailable frames.
    pub reason: Option<&'static str>,
}

/// Immutable bounded report data without session-scoped variable handles.
#[derive(Debug)]
pub struct DebugSnapshot {
    /// Globals in declaration order, captured once.
    pub globals: Vec<FailureGlobal>,
    /// Tasks in capture-priority order.
    pub tasks: Vec<DebugTaskSnapshot>,
    /// Expression values or independent failures in request order.
    pub expressions: Vec<Result<FailureValue, DebugSessionError>>,
    /// Selected task, or no task for completed globals-only evaluation.
    pub selected_task: Option<u64>,
    /// Selected frame depth, or no frame at normal completion.
    pub selected_frame: Option<usize>,
    /// Invalid explicit context, including requests with no expressions.
    pub selection_error: Option<DebugSessionError>,
    /// Full catalog size before report bounds.
    pub total_tasks: usize,
    /// Full image global count before report bounds.
    pub total_globals: usize,
    /// Explicit capture omissions.
    pub reasons: BTreeSet<&'static str>,
}

impl DebugSession {
    /// Captures current state and non-executing expressions without resuming bytecode.
    /// Completed sessions expose globals only. Calls and other executing IR are rejected.
    pub fn capture_snapshot(
        &mut self,
        expressions: &[DebugExpression],
        selected_task: Option<u64>,
        frame: Option<usize>,
    ) -> DebugSnapshot {
        let _timer = super::profile::inspection();
        let completed = self.state == DebugSessionState::Terminated;
        if completed && let Some(worker) = self.runtime.worker(0) {
            self.inspection_generation = self.inspection_generation.wrapping_add(1).max(1);
            self.inspections.insert(
                0,
                InspectionSnapshot::completed(
                    worker,
                    self.inspection_generation,
                    self.inspection_limits,
                ),
            );
            self.inspection_task_id = 0;
        }
        if self.last_stop.reason == DebugStopReason::RuntimeError
            && let Some(worker) = self.runtime.worker(self.last_stop.task_id)
        {
            self.inspection_generation = self.inspection_generation.wrapping_add(1).max(1);
            self.inspections.insert(
                self.last_stop.task_id,
                InspectionSnapshot::capture_fault(
                    worker,
                    self.inspection_generation,
                    self.inspection_limits,
                ),
            );
        }
        let task_id = selected_task.unwrap_or(self.last_stop.task_id);
        let depth = frame.unwrap_or(0);
        let context = if completed {
            if selected_task.is_some() || frame.is_some() {
                Err(unknown_task(task_id))
            } else {
                Ok(None)
            }
        } else {
            self.inspections
                .get(&task_id)
                .ok_or_else(|| unknown_task(task_id))
                .and_then(|inspection| inspection.stack(depth, 1))
                .and_then(|page| {
                    page.items
                        .first()
                        .map(|frame| Some(frame.id))
                        .ok_or_else(|| DebugSessionError {
                            kind: DebugErrorKind::UnknownFrame,
                            message: format!(
                                "Frame depth {depth} is unavailable in task {task_id}."
                            ),
                            hint: "Select a depth in the captured stack.".into(),
                        })
                })
        };
        if !completed {
            self.inspection_task_id = task_id;
        }
        let mut result = DebugSnapshot {
            globals: Vec::new(),
            tasks: Vec::new(),
            expressions: Vec::new(),
            selected_task: (!completed).then_some(task_id),
            selected_frame: (!completed).then_some(depth),
            selection_error: context.as_ref().err().cloned(),
            total_tasks: 0,
            total_globals: self.executable.executable().globals.len(),
            reasons: BTreeSet::new(),
        };
        let mut budget = ValueBudget::new(65_536, 4 * 1024 * 1024);
        for expression in expressions {
            let captured = context.clone().and_then(|frame_id| {
                if !non_executing(expression) {
                    return Err(DebugSessionError {
                        kind: DebugErrorKind::ForbiddenCallEffect,
                        message: "Batch expressions cannot execute program code.".into(),
                        hint: "Use names, stored fields, indexes or scalar operators.".into(),
                    });
                }
                let inspection = self
                    .inspections
                    .get(&self.inspection_task_id)
                    .ok_or_else(|| unknown_task(task_id))?;
                inspection.validate_evaluation_frame(frame_id)?;
                let value = crate::vm::debug::evaluation::evaluate_value(
                    expression,
                    DebugEvaluationLimits::default(),
                    |name| inspection.resolve_evaluation_name(frame_id, name),
                    |_, _| {
                        Err(DebugSessionError {
                            kind: DebugErrorKind::ForbiddenCallEffect,
                            message:
                                "Batch expressions cannot invoke constructors, getters or routines."
                                    .into(),
                            hint: "Inspect a stored value instead.".into(),
                        })
                    },
                )?;
                let retained = RetainedValue {
                    domain: Some(inspection.domain.clone()),
                    name: String::new(),
                    type_name: value.type_name().into(),
                    value: Some(value),
                    presentation_hint: None,
                    depth: 0,
                    visited_cells: HashSet::new(),
                    debug_type: None,
                    origin: (),
                };
                snapshot::capture(
                    &retained,
                    LIMITS,
                    Some(self.executable.executable()),
                    &mut budget,
                )
                .ok_or_else(|| DebugSessionError {
                    kind: DebugErrorKind::InspectionLimit,
                    message: "Expression capture exceeded the snapshot budget.".into(),
                    hint: "Request fewer or smaller values.".into(),
                })
            });
            result.expressions.push(captured);
        }
        let mut tasks = self.runtime.catalog();
        result.total_tasks = tasks.len();
        tasks.sort_by_key(|task| {
            (
                if task.id == self.last_stop.task_id {
                    0
                } else if Some(task.id) == selected_task {
                    1
                } else {
                    2
                },
                task.id,
            )
        });
        let mut frames_left = 256_usize;
        let mut bindings_left = 4096_usize;
        for task in tasks.into_iter().take(64) {
            let mut captured = DebugTaskSnapshot {
                task,
                frames: Vec::new(),
                total_frames: None,
                reason: None,
            };
            if !completed
                && self.runtime.task_is_inspectable(captured.task.id)
                && let Some(worker) = self.runtime.worker(captured.task.id)
            {
                let count = frames::visit(
                    worker,
                    if self.last_stop.reason == DebugStopReason::RuntimeError
                        && captured.task.id == self.last_stop.task_id
                    {
                        Boundary::Fault
                    } else {
                        Boundary::Stopped
                    },
                    frames_left.min(32),
                    |owner, frame, depth| {
                        if let Some(frame) = detached_frame::capture(
                            owner,
                            frame,
                            depth,
                            &mut budget,
                            LIMITS,
                            bindings_left,
                        ) {
                            bindings_left = bindings_left.saturating_sub(frame.bindings.len());
                            captured.frames.push(frame);
                        } else {
                            captured.reason = Some(budget.exhausted.unwrap_or("value_limit"));
                        }
                    },
                );
                captured.total_frames = count.total;
                captured.reason = captured.reason.or(count.unavailable_reason);
                if count
                    .total
                    .is_some_and(|total| total > captured.frames.len())
                {
                    captured.reason = captured.reason.or(Some("frame_limit"));
                }
                frames_left = frames_left.saturating_sub(captured.frames.len());
            } else {
                captured.total_frames = Some(0);
                captured.reason = Some("no_live_frame");
            }
            if let Some(reason) = captured.reason.filter(|reason| *reason != "no_live_frame") {
                result.reasons.insert(reason);
            }
            for frame in &captured.frames {
                if let Some(reason) = frame.reason {
                    result.reasons.insert(reason);
                }
            }
            result.tasks.push(captured);
        }
        if result.tasks.len() < result.total_tasks {
            result.reasons.insert("task_limit");
        }
        if let Some(worker) = self.runtime.worker(0) {
            let image = worker.executable.executable();
            let slots = globals::slots(worker);
            for (index, global) in image.globals.iter().take(bindings_left).enumerate() {
                let (value, unavailable) = match &slots {
                    Ok(slots) => globals::read_slot(slots, index),
                    Err(reason) => ReadValue::Unavailable(reason),
                }
                .into_parts();
                let retained = RetainedValue {
                    domain: Some(worker.domain.clone()),
                    name: String::new(),
                    type_name: value
                        .as_ref()
                        .map_or("dynamic", |value| value.type_name())
                        .into(),
                    value,
                    presentation_hint: unavailable.map(|reason| format!("unavailable: {reason}")),
                    depth: 0,
                    visited_cells: HashSet::new(),
                    debug_type: Some(global.ty),
                    origin: (),
                };
                let Some(value) = snapshot::capture(&retained, LIMITS, Some(image), &mut budget)
                else {
                    result.reasons.insert("value_limit");
                    break;
                };
                result.globals.push(FailureGlobal {
                    name: image
                        .strings
                        .get(global.name)
                        .unwrap_or("<global>")
                        .chars()
                        .take(4096)
                        .collect(),
                    value,
                });
            }
        }
        if result.globals.len() < result.total_globals {
            result.reasons.insert("binding_limit");
        }
        for global in &result.globals {
            collect_reasons(&global.value, &mut result.reasons);
        }
        for task in &result.tasks {
            for frame in &task.frames {
                for binding in &frame.bindings {
                    collect_reasons(&binding.value, &mut result.reasons);
                }
            }
        }
        for value in result.expressions.iter().flatten() {
            collect_reasons(value, &mut result.reasons);
        }
        result
    }
}

/// Checks the existing expression IR without admitting any operation that invokes code.
pub fn non_executing(expression: &DebugExpression) -> bool {
    match expression {
        DebugExpression::Integer(_)
        | DebugExpression::Real(_)
        | DebugExpression::Boolean(_)
        | DebugExpression::String(_)
        | DebugExpression::Name(_) => true,
        DebugExpression::Unary { operand, .. } => non_executing(operand),
        DebugExpression::Binary { left, right, .. } => non_executing(left) && non_executing(right),
        DebugExpression::Field { base, .. } => non_executing(base),
        DebugExpression::Index { base, index } => non_executing(base) && non_executing(index),
        _ => false,
    }
}

fn collect_reasons(value: &FailureValue, reasons: &mut BTreeSet<&'static str>) {
    if let Some(reason) = value.reason {
        reasons.insert(reason);
    }
    for (_, child) in &value.children {
        collect_reasons(child, reasons);
    }
}
