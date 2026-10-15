//! Bounded pre-unwind frame and binding capture through the shared inspection core.

use super::{CaptureState, FailureBinding, FailureFrame, FailureSnapshot, MAX_FRAMES, MAX_STRING};
use crate::vm::{
    inspection::{
        bindings,
        frames::{self, Boundary, Frame},
        location,
        values::{
            ValueLimits,
            budget::ValueBudget,
            render::{self, RetainedValue},
            snapshot,
        },
    },
    worker::Worker,
};
use std::sync::Arc;
use std::{
    collections::HashSet,
    panic::{AssertUnwindSafe, catch_unwind},
};

const LIMITS: ValueLimits = ValueLimits {
    max_children: 256,
    max_string_chars: MAX_STRING,
    max_summary_chars: MAX_STRING,
    max_depth: 16,
};

impl CaptureState {
    pub(super) fn capture(self: &Arc<Self>, worker: &Worker, snapshot: &mut FailureSnapshot) {
        let (mut usage, mut budget) = self.budget();
        let before = (budget.values, budget.bytes);
        let offset = snapshot.frames.len();
        // Capture is diagnostic work. A failed capture must not replace the program's error
        // or leave partially retained data outside the shared budget.
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut captured = Vec::new();
            let mut omission = None;
            if offset == 0 && !budget.charge(std::mem::size_of::<FailureSnapshot>()) {
                return (captured, None, budget.exhausted);
            }
            let count = frames::visit(
                worker,
                Boundary::Continuation,
                MAX_FRAMES.saturating_sub(offset),
                |owner, frame, depth| {
                    if omission.is_some() {
                        return;
                    }
                    match capture_frame(owner, frame, offset + depth, &mut budget) {
                        Some(frame) => captured.push(frame),
                        None => omission = Some(budget.exhausted.unwrap_or("missing_metadata")),
                    }
                },
            );
            let reason = omission.or(count.unavailable_reason).or_else(|| {
                count
                    .total
                    .filter(|total| total.saturating_add(offset) > MAX_FRAMES)
                    .map(|_| "frame_limit")
            });
            (captured, count.total, reason)
        }));
        match result {
            Ok((frames, total, reason)) => {
                snapshot.lease.charge(self, &mut usage, before, &budget);
                snapshot.frames.extend(frames);
                snapshot.total_frames = if offset == 0 {
                    total
                } else {
                    snapshot
                        .total_frames
                        .zip(total)
                        .map(|(a, b)| a.saturating_add(b))
                };
                snapshot.reason = snapshot
                    .reason
                    .filter(|reason| *reason != "no_live_frame")
                    .or(reason);
            }
            Err(_) => snapshot.reason = Some("capture_failed"),
        }
    }
}

fn capture_frame(
    worker: &Worker,
    frame: Frame,
    depth: usize,
    budget: &mut ValueBudget,
) -> Option<FailureFrame> {
    if !budget.charge(std::mem::size_of::<FailureFrame>()) {
        return None;
    }
    let image = worker.executable.executable();
    let function = image.functions.get(usize::from(frame.function.get()))?;
    let (name, truncated) = budget.text(
        location::routine_name(image.strings.get(function.name).unwrap_or("<function>")),
        MAX_STRING,
    );
    let mut result = FailureFrame {
        depth,
        function: name,
        location: location::position_at(image, frame.function, frame.instruction),
        bindings: Vec::new(),
        reason: truncated.then_some("string_limit"),
    };
    if result.location.is_none() || function.debug.scopes.is_empty() {
        result.reason = Some("missing_metadata");
    }
    let scope = location::scope_at(image, frame.function, frame.instruction).unwrap_or(0);
    let visible = location::visible_scope_chain(function, scope);
    for binding in function
        .debug
        .bindings
        .iter()
        .filter(|binding| !binding.hidden && visible.contains(&binding.scope))
    {
        if !budget.charge(std::mem::size_of::<FailureBinding>()) {
            result.reason = budget.exhausted;
            break;
        }
        let (name, truncated) = budget.text(
            image.strings.get(binding.name).unwrap_or("<binding>"),
            MAX_STRING,
        );
        if truncated {
            result.reason = Some(budget.exhausted.unwrap_or("string_limit"));
        }
        let (value, unavailable) = bindings::read(worker, frame.base, binding).into_parts();
        let type_name = image.strings.get(binding.type_name).unwrap_or("dynamic");
        let type_prefix = render::metadata(type_name);
        if type_prefix.len() < type_name.len() {
            result.reason = Some("string_limit");
        }
        let retained = RetainedValue {
            name: String::new(),
            value,
            type_name: type_prefix.to_string(),
            presentation_hint: unavailable.map(|reason| format!("unavailable: {reason}")),
            depth: 0,
            visited_cells: HashSet::new(),
            debug_type: Some(bindings::value_type(worker, binding)),
            origin: (),
        };
        let Some(value) = snapshot::capture(&retained, LIMITS, Some(image), budget) else {
            result.reason = Some(budget.exhausted.unwrap_or("value_limit"));
            break;
        };
        result.bindings.push(FailureBinding {
            name,
            kind: binding.kind,
            scope_id: binding.scope,
            value,
        });
    }
    Some(result)
}
