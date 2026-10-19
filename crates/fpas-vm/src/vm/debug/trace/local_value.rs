//! Nonblocking scalar projection; no aggregate root survives a source read.

use super::model::{DebugTracePathStep, TraceLeaf};
use crate::vm::{
    inspection::bindings::{self, ReadValue},
    worker::Worker,
};
use fpas_bytecode::{DebugType, DebugTypeId, Executable, Value};

pub(super) fn binding_value(
    worker: &Worker,
    base: usize,
    binding: &fpas_bytecode::DebugBinding,
    watch: &super::DebugTraceWatch,
    parameter: Option<&super::local_state::ParameterValue>,
) -> Option<TraceLeaf> {
    use super::local_state::ParameterValue;
    match parameter {
        Some(ParameterValue::Frozen(value)) => value.clone(),
        Some(ParameterValue::Alias(value)) => read(worker, value, watch.root_ty, &watch.path),
        None => {
            let register = base + usize::from(binding.register.get());
            if !worker.register_is_initialized(register) {
                return None;
            }
            read(
                worker,
                worker.registers.get(register)?,
                watch.root_ty,
                &watch.path,
            )
        }
    }
}

pub(super) fn read(
    worker: &Worker,
    root: &Value,
    ty: DebugTypeId,
    fields: &[DebugTracePathStep],
) -> Option<TraceLeaf> {
    dereference(worker, root, ty, fields, 0)
}

fn dereference(
    worker: &Worker,
    root: &Value,
    ty: DebugTypeId,
    fields: &[DebugTracePathStep],
    depth: usize,
) -> Option<TraceLeaf> {
    if depth > 16 {
        return None;
    }
    match root {
        Value::Cell(cell) => {
            let value = cell.try_lock().ok()?;
            dereference(worker, &value, ty, fields, depth + 1)
        }
        Value::Reference(reference) => match bindings::read_reference(worker, reference) {
            ReadValue::Available(value) => dereference(worker, &value, ty, fields, depth + 1),
            _ => None,
        },
        value => project(worker.executable.executable(), value, ty, fields),
    }
}

pub(super) fn project(
    image: &Executable,
    mut value: &Value,
    mut ty: DebugTypeId,
    fields: &[DebugTracePathStep],
) -> Option<TraceLeaf> {
    for step in fields {
        match step {
            DebugTracePathStep::Field { slot } => {
                let DebugType::Record(expected) = image.debug_types.get(ty.get() as usize)? else {
                    return None;
                };
                let Value::Record(record) = value else {
                    return None;
                };
                if record.body().layout.record != *expected {
                    return None;
                }
                ty = image
                    .records
                    .get(expected.get() as usize)?
                    .fields
                    .get(*slot)?
                    .ty;
                value = record.body().values.get(*slot)?;
            }
            DebugTracePathStep::Index { index } => {
                let DebugType::Array(element) = image.debug_types.get(ty.get() as usize)? else {
                    return None;
                };
                let Value::Array(values) = value else {
                    return None;
                };
                ty = *element;
                let Some(child) = usize::try_from(*index).ok().and_then(|n| values.get(n)) else {
                    return Some(TraceLeaf::Absent);
                };
                value = child;
            }
        }
    }
    let valid = match (image.debug_types.get(ty.get() as usize)?, value) {
        (DebugType::Enum(expected), Value::Enum(value)) => {
            let actual = &value.body().layout;
            actual.enumeration == *expected
                && image
                    .enum_variants
                    .get(actual.variant_id.get() as usize)
                    .is_some_and(|variant| variant.owner == *expected)
        }
        (DebugType::Integer, Value::Integer(_))
        | (DebugType::Real, Value::Real(_))
        | (DebugType::Boolean, Value::Boolean(_))
        | (DebugType::String, Value::Str(_)) => true,
        _ => false,
    };
    valid.then(|| TraceLeaf::Value(value.clone()))
}
