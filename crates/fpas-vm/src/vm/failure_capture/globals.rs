//! Read-only globals from the actual root storage before worker disposal.

use super::{CaptureState, FailureGlobal, FailureGlobals};
use crate::vm::{
    inspection::{
        bindings::ReadValue,
        globals,
        values::{render::RetainedValue, snapshot},
    },
    worker::Worker,
};
use std::{collections::HashSet, sync::Arc};

impl CaptureState {
    /// Capture globals after primary values, using worker-local slots where enabled.
    pub(in crate::vm) fn capture_globals(self: &Arc<Self>, worker: &Worker) {
        let header = self
            .reserve_model(std::mem::size_of::<FailureGlobals>())
            .unwrap_or_else(|| self.reserve_terminal(std::mem::size_of::<FailureGlobals>()));
        let (mut usage, mut budget) = self.budget();
        let before = (budget.values, budget.bytes);
        let image = worker.executable.executable();
        let slots = globals::slots(worker);
        let mut result = FailureGlobals {
            bindings: Vec::new(),
            total: image.globals.len(),
            reason: if image.globals.is_empty() {
                None
            } else {
                slots.as_ref().err().copied()
            },
            _lease: header,
        };
        for (index, global) in image.globals.iter().enumerate() {
            if !budget.charge(std::mem::size_of::<FailureGlobal>()) {
                result.reason = budget.exhausted;
                break;
            }
            let (name, truncated) = budget.text(
                image.strings.get(global.name).unwrap_or("<global>"),
                super::MAX_STRING,
            );
            if truncated {
                result.reason = Some(budget.exhausted.unwrap_or("string_limit"));
            }
            let (value, unavailable) = match &slots {
                Ok(slots) => globals::read_slot(slots, index),
                Err(reason) => ReadValue::Unavailable(reason),
            }
            .into_parts();
            let retained = RetainedValue {
                name: String::new(),
                type_name: super::super::inspection::values::render::metadata(
                    value.as_ref().map_or("dynamic", |value| value.type_name()),
                )
                .to_string(),
                value,
                presentation_hint: unavailable.map(|reason| format!("unavailable: {reason}")),
                depth: 0,
                visited_cells: HashSet::new(),
                debug_type: Some(global.ty),
                origin: (),
            };
            let Some(value) =
                snapshot::capture(&retained, super::capture::LIMITS, Some(image), &mut budget)
            else {
                result.reason = Some(budget.exhausted.unwrap_or("value_limit"));
                break;
            };
            result.bindings.push(FailureGlobal { name, value });
        }
        result._lease.charge(self, &mut usage, before, &budget);
        drop(usage);
        *self.root_globals.lock().unwrap_or_else(|p| p.into_inner()) = Some(Arc::new(result));
    }
}
