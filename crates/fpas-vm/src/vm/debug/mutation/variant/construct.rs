//! Exact-field validation and detached complete-variant construction.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::sync::Arc;

use fpas_bytecode::{DebugType, Value, VerifiedExecutable};

use super::super::super::evaluation::{DebugEvaluationLimits, DebugExpression};
use super::super::super::types::DebugSessionError;
use super::diagnostics::{
    duplicate_field, extra_fields, identity_bearing_field, missing_fields, unsupported_metadata,
};
use super::model::{DebugVariantDescription, VariantKind, VariantMetadata, WrapperMetadata};

/// Describe only variants whose declared fields can enter complete construction.
pub(in crate::vm::debug) fn constructible_description(
    executable: &fpas_bytecode::Executable,
    wrapper: &WrapperMetadata,
) -> Result<DebugVariantDescription, DebugSessionError> {
    let mut constructible = wrapper.clone();
    constructible.variants.clear();
    for variant in &wrapper.variants {
        match require_constructible_fields(executable, variant) {
            Ok(()) => constructible.variants.push(variant.clone()),
            Err(error)
                if error.kind == crate::vm::debug::types::DebugErrorKind::VariableValueType => {}
            Err(error) => return Err(error),
        }
    }
    Ok(constructible.description())
}

/// Return field expressions in declaration order after an exact name-set check.
pub(in crate::vm::debug) fn ordered_field_expressions<'a>(
    variant: &VariantMetadata,
    fields: &'a [(String, DebugExpression)],
) -> Result<Vec<&'a DebugExpression>, DebugSessionError> {
    let mut used = vec![None; variant.fields.len()];
    for (name, expression) in fields {
        let matches = variant
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.name.eq_ignore_ascii_case(name))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let index = match matches.as_slice() {
            [index] => *index,
            [] => return Err(extra_fields(variant, &[name.as_str()])),
            _ => return Err(duplicate_field(variant, name)),
        };
        if used[index].is_some() {
            return Err(duplicate_field(variant, name));
        }
        used[index] = Some(expression);
    }
    let missing = variant
        .fields
        .iter()
        .zip(&used)
        .filter_map(|(field, expression)| expression.is_none().then_some(field.name.as_str()))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(missing_fields(variant, &missing));
    }
    Ok(used.into_iter().flatten().collect())
}

/// Reject function, task, and capture-cell fields before any field evaluation.
pub(in crate::vm::debug) fn require_constructible_fields(
    executable: &fpas_bytecode::Executable,
    variant: &VariantMetadata,
) -> Result<(), DebugSessionError> {
    for field in &variant.fields {
        match executable.debug_types.get(field.ty.get() as usize) {
            Some(DebugType::Function { .. } | DebugType::Cell(_) | DebugType::Task(_)) => {
                return Err(identity_bearing_field(variant, &field.name));
            }
            Some(_) => {}
            None => return Err(unsupported_metadata()),
        }
    }
    Ok(())
}

/// Build one complete detached enum, `Result`, or `Option` value.
pub(in crate::vm::debug) fn complete_value(
    executable: &VerifiedExecutable,
    variant: &VariantMetadata,
    mut values: Vec<Value>,
    limits: DebugEvaluationLimits,
) -> Result<Value, DebugSessionError> {
    if values.len() != variant.fields.len() {
        return Err(unsupported_metadata());
    }
    match &variant.kind {
        VariantKind::OptionNone => Ok(Value::OptionNone),
        VariantKind::OptionSome => Ok(Value::option_some(take_single(&mut values)?)),
        VariantKind::ResultOk => Ok(Value::result_ok(take_single(&mut values)?)),
        VariantKind::ResultError => Ok(Value::result_error(take_single(&mut values)?)),
        VariantKind::Enum { layout } => crate::vm::debug::construct_enum(
            executable,
            Arc::clone(layout),
            values,
            limits.max_depth,
            limits.max_detached_values,
        ),
    }
}

fn take_single(values: &mut Vec<Value>) -> Result<Value, DebugSessionError> {
    if values.len() == 1 {
        Ok(values.remove(0))
    } else {
        Err(unsupported_metadata())
    }
}
