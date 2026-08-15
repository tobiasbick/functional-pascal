//! Zero-capture and capturing named-routine proof and function-value materialization.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod captures;

use std::collections::HashSet;

use fpas_bytecode::{
    DebugBindingKind, DebugTypeId, Executable, FunctionId, FunctionInfo, Value, VerifiedExecutable,
};

use super::super::super::evaluation::DebugEvaluationLimits;
use super::super::super::inspection::{InspectionSnapshot, MutationTarget};
use super::super::super::routines::matching_functions;
use super::super::super::types::{DebugErrorKind, DebugSessionError};
use super::signature;

/// Materialize one canonical function value for a catalog routine.
#[expect(
    clippy::too_many_arguments,
    reason = "named-routine preparation keeps catalog, owner-frame, task, and destination proofs together"
)]
pub(in crate::vm::debug) fn prepare(
    executable: &VerifiedExecutable,
    inspection: Option<&InspectionSnapshot>,
    name: &str,
    expected: DebugTypeId,
    frame_id: Option<u64>,
    task_id: u64,
    destination: Option<&MutationTarget>,
    limits: DebugEvaluationLimits,
) -> Result<Value, DebugSessionError> {
    if matches!(
        executable
            .executable()
            .debug_types
            .get(expected.get() as usize),
        Some(fpas_bytecode::DebugType::Dynamic)
    ) {
        return Err(type_error(
            "destination is a Dynamic endpoint",
            "Assign between bindings with declared function signatures, not Dynamic storage.",
        ));
    }
    let (function_id, canonical) = resolve_unique(executable, name)?;
    let function = executable
        .executable()
        .functions
        .get(function_id.get() as usize)
        .ok_or_else(|| {
            type_error(
                "resolved routine is missing from executable metadata",
                "Assign a routine that is present in the current executable.",
            )
        })?;
    if function.capture_count != 0 {
        let inspection = inspection.ok_or_else(|| {
            type_error(
                &format!("routine `{canonical}` requires captures"),
                "Select the live frame of the nested routine's enclosing function, then assign the routine name.",
            )
        })?;
        let constructed = captures::materialize(
            executable,
            inspection,
            function_id,
            function,
            &canonical,
            frame_id,
            task_id,
            destination,
            limits,
        )?;
        let (parameters, result) =
            portable_signature(executable.executable(), function, &canonical)?;
        signature::require_signature(
            &executable.executable().debug_types,
            &parameters,
            result,
            expected,
            limits.max_depth,
            limits.max_detached_values,
        )?;
        return Ok(constructed);
    }
    let (parameters, result) = portable_signature(executable.executable(), function, &canonical)?;
    signature::require_signature(
        &executable.executable().debug_types,
        &parameters,
        result,
        expected,
        limits.max_depth,
        limits.max_detached_values,
    )?;
    Ok(Value::function(function_id, canonical, Vec::new()))
}

fn resolve_unique(
    executable: &VerifiedExecutable,
    name: &str,
) -> Result<(FunctionId, String), DebugSessionError> {
    let matches = matching_functions(executable, name);
    match matches.as_slice() {
        [function_id] => {
            let canonical = executable
                .executable()
                .functions
                .get(function_id.get() as usize)
                .and_then(|function| executable.executable().strings.get(function.name))
                .ok_or_else(|| {
                    type_error(
                        "resolved routine does not retain a canonical name",
                        "Assign a routine whose executable name is present in metadata.",
                    )
                })?;
            Ok((*function_id, canonical.to_string()))
        }
        [] => Err(DebugSessionError {
            kind: DebugErrorKind::UnknownName,
            message: format!("debug function assignment source `{name}` is not a visible binding or executable routine"),
            hint: "Use a visible function binding such as `Backup`, or a unique routine name such as `AddTwo`."
                .to_string(),
        }),
        _ => Err(DebugSessionError {
            kind: DebugErrorKind::AmbiguousCallable,
            message: format!("debug function assignment source `{name}` matches multiple executable routines"),
            hint: "Use a fully qualified routine name that identifies one function.".to_string(),
        }),
    }
}

fn portable_signature(
    executable: &Executable,
    function: &FunctionInfo,
    canonical: &str,
) -> Result<(Vec<DebugTypeId>, DebugTypeId), DebugSessionError> {
    if function
        .debug
        .bindings
        .iter()
        .any(|binding| binding.kind == DebugBindingKind::Parameter && binding.hidden)
    {
        return Err(type_error(
            &format!("routine `{canonical}` has hidden parameter metadata"),
            "Assign a routine whose parameters are ordinary declared source parameters.",
        ));
    }
    let parameters = function
        .debug
        .bindings
        .iter()
        .filter(|binding| binding.kind == DebugBindingKind::Parameter && !binding.hidden)
        .collect::<Vec<_>>();
    let arity = usize::from(function.arity);
    if parameters.len() != arity {
        return Err(type_error(
            &format!("routine `{canonical}` does not retain {arity} portable parameter(s)"),
            "Assign a routine whose debug metadata includes every declared parameter.",
        ));
    }
    let mut seen_registers = HashSet::new();
    for (index, binding) in parameters.iter().enumerate() {
        if !seen_registers.insert(binding.register.get()) {
            return Err(type_error(
                &format!("routine `{canonical}` parameter registers are incomplete or duplicated"),
                "Assign a routine whose parameters retain unique debug registers.",
            ));
        }
        if executable
            .debug_types
            .get(binding.ty.get() as usize)
            .is_none()
        {
            return Err(type_error(
                &format!(
                    "routine `{canonical}` parameter {} has no portable type",
                    index.saturating_add(1)
                ),
                "Assign a routine whose parameter types are retained in executable metadata.",
            ));
        }
    }
    let result = function.debug.result_type.ok_or_else(|| {
        type_error(
            &format!("routine `{canonical}` does not retain a portable result type"),
            "Assign a routine whose result type is present in debug metadata.",
        )
    })?;
    if executable.debug_types.get(result.get() as usize).is_none() {
        return Err(type_error(
            &format!("routine `{canonical}` result type is not a valid portable type"),
            "Assign a routine whose result type is retained in executable metadata.",
        ));
    }
    Ok((
        parameters.into_iter().map(|binding| binding.ty).collect(),
        result,
    ))
}

pub(super) fn type_error(detail: &str, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug function assignment is rejected: {detail}"),
        hint: hint.to_string(),
    }
}

#[cfg(test)]
#[path = "routine/tests.rs"]
mod tests;
