//! Shared detached frame/binding capture for stopped sessions and original failures.

use super::{
    bindings,
    frames::Frame,
    location,
    values::{
        ValueLimits,
        budget::ValueBudget,
        render::{self, RetainedValue},
        snapshot,
    },
};
use crate::execution::vm::{
    failure_capture::{FailureBinding, FailureFrame},
    worker::Worker,
};
use std::collections::HashSet;
const MAX_STRING: usize = 4096;

/// Shared diagnostic frame capture with explicit traversal and binding bounds.
pub(in crate::execution::vm) fn capture(
    worker: &Worker,
    frame: Frame,
    depth: usize,
    budget: &mut ValueBudget,
    limits: ValueLimits,
    max_bindings: usize,
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
        if result.bindings.len() >= max_bindings {
            result.reason = Some("binding_limit");
            break;
        }
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
            domain: Some(worker.domain.clone()),
            name: String::new(),
            value,
            type_name: type_prefix.to_string(),
            presentation_hint: unavailable.map(|reason| format!("unavailable: {reason}")),
            depth: 0,
            visited_cells: HashSet::new(),
            debug_type: Some(bindings::value_type(worker, binding)),
            origin: (),
        };
        let Some(value) = snapshot::capture(&retained, limits, Some(image), budget) else {
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
