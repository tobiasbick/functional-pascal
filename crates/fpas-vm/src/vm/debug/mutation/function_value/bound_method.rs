//! Metadata-backed construction of first-class bound record methods.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{DebugType, DebugTypeId, FunctionId, Value, VerifiedExecutable};

use super::super::super::evaluation::DebugEvaluationLimits;
use super::super::super::routines::matching_functions;
use super::super::super::types::{DebugErrorKind, DebugSessionError};
use super::{captures, routine, signature};

/// Construct a method value from an exact record layout member mapping.
pub(in crate::vm::debug) fn prepare(
    executable: &VerifiedExecutable,
    receiver: Value,
    member: &str,
    expected: DebugTypeId,
    limits: DebugEvaluationLimits,
) -> Result<Value, DebugSessionError> {
    let Value::Record(record) = &receiver else {
        return Err(type_error(
            &format!(
                "bound receiver has type {}, not record",
                receiver.type_name()
            ),
            "Use `recordValue.Method` where the receiver evaluates to a record.",
        ));
    };
    let image = executable.executable();
    let record_id = record.body().layout.record;
    let layout = image
        .records
        .get(usize::from(record_id.get()))
        .ok_or_else(|| {
            type_error(
                "bound receiver references a missing record layout",
                "Rebuild the executable with the current compiler.",
            )
        })?;
    let mut matching_methods = layout.methods.iter().filter(|method| {
        image
            .strings
            .get(method.name)
            .is_some_and(|name| name.eq_ignore_ascii_case(member))
    });
    let method = matching_methods.next().ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::UnknownName,
            message: format!(
                "record `{}` has no bound method `{member}` in executable metadata",
                record.body().layout.type_name
            ),
            hint: "Use an instance method retained by the current executable, or rebuild the program with the current compiler."
                .to_string(),
        })?;
    if matching_methods.next().is_some() {
        return Err(type_error(
            &format!(
                "record `{}` has duplicate method metadata for `{member}`",
                record.body().layout.type_name
            ),
            "Rebuild the executable with unique record member metadata.",
        ));
    }
    let canonical = image.strings.get(method.routine).ok_or_else(|| {
        type_error(
            "bound method metadata references a missing canonical routine name",
            "Rebuild the executable with the current compiler.",
        )
    })?;
    let function_id = resolve_exact(executable, canonical)?;
    let function = &image.functions[usize::from(function_id.get())];
    if function.capture_count != 0 {
        return Err(type_error(
            &format!("method `{canonical}` unexpectedly requires lexical captures"),
            "Use an ordinary record instance method whose receiver is its only bound value.",
        ));
    }
    let (parameters, result) = routine::portable_signature(image, function, canonical)?;
    let Some((receiver_type, visible_parameters)) = parameters.split_first() else {
        return Err(type_error(
            &format!("method `{canonical}` has no receiver parameter"),
            "Rebuild the executable with complete method parameter metadata.",
        ));
    };
    if !matches!(
        image.debug_types.get(receiver_type.get() as usize),
        Some(DebugType::Record(found)) if *found == record_id
    ) {
        return Err(type_error(
            &format!("method `{canonical}` receiver type does not match the runtime record layout"),
            "Use a method declared on the receiver's exact record type.",
        ));
    }
    signature::require_signature(
        &image.debug_types,
        visible_parameters,
        result,
        expected,
        limits.max_depth,
        limits.max_detached_values,
    )?;
    let value = Value::bound_function(function_id, canonical.to_string(), receiver);
    let Value::Function(bound) = &value else {
        unreachable!("bound function constructor must return a function value")
    };
    captures::require_eligible(bound, limits.max_depth, limits.max_detached_values)?;
    Ok(value)
}

fn resolve_exact(
    executable: &VerifiedExecutable,
    canonical: &str,
) -> Result<FunctionId, DebugSessionError> {
    match matching_functions(executable, canonical).as_slice() {
        [function] => Ok(*function),
        [] => Err(type_error(
            &format!("method routine `{canonical}` is missing from the executable"),
            "Rebuild the executable so its record and function metadata agree.",
        )),
        _ => Err(type_error(
            &format!("method routine identity `{canonical}` is not unique"),
            "Rebuild the executable with unique canonical routine identities.",
        )),
    }
}

fn type_error(detail: &str, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug bound-method assignment is rejected: {detail}"),
        hint: hint.to_string(),
    }
}
