//! Group selected leaves under one declaration lifetime and project ABI arguments early.

use super::{
    TraceHistory,
    local_state::{LiveLocal, LocalFrame, ParameterValue},
};
use crate::vm::worker::Worker;
use fpas_bytecode::{DebugBindingKind, DebugType, Value};
use std::collections::BTreeMap;

pub(super) fn admit(
    history: &mut TraceHistory,
    worker: &Worker,
    frame: &mut LocalFrame,
    paths: Vec<(usize, usize)>,
) {
    let info = &worker.executable.executable().functions[frame.function.get() as usize];
    let mut instances = BTreeMap::new();
    for (watch, binding) in paths {
        let instance = if let Some(instance) = instances.get(&binding) {
            *instance
        } else {
            let Some(instance) = history.declaration() else {
                return;
            };
            instances.insert(binding, instance);
            instance
        };
        if !history.instance(watch, frame.activation, instance) {
            return;
        }
        let parameter = if info.debug.bindings[binding].kind != DebugBindingKind::Local {
            let kind = info.debug.bindings[binding].kind;
            let ordinal = info.debug.bindings[..binding]
                .iter()
                .filter(|b| b.kind == kind)
                .count()
                + if kind == DebugBindingKind::Capture {
                    usize::from(info.arity)
                } else {
                    0
                };
            let value = worker.registers.get(frame.base + ordinal).filter(|value| {
                let source = &info.debug.bindings[binding];
                if kind != DebugBindingKind::Capture {
                    return true;
                }
                if source.cell_backed {
                    return matches!(value, Value::Cell(_));
                }
                if matches!(
                    worker
                        .executable
                        .executable()
                        .debug_types
                        .get(source.ty.get() as usize),
                    Some(DebugType::Reference(_))
                ) {
                    matches!(value, Value::Reference(_))
                } else {
                    !matches!(value, Value::Cell(_) | Value::Reference(_))
                }
            });
            Some(match value {
                Some(value @ (Value::Cell(_) | Value::Reference(_))) => {
                    ParameterValue::Alias(value.clone())
                }
                other => {
                    let selected = &history.report.watches[watch];
                    ParameterValue::Frozen(other.and_then(|root| {
                        super::local_value::read(worker, root, &selected.path_types, &selected.path)
                    }))
                }
            })
        } else {
            None
        };
        frame.live.push(LiveLocal {
            watch,
            binding,
            instance,
            parameter,
        });
    }
}
