//! Materialize immutable value captures from the selected lexical-owner frame.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{DebugCaptureKind, FunctionId, FunctionInfo, Value, VerifiedExecutable};

use super::super::super::super::evaluation::DebugEvaluationLimits;
use super::super::super::super::inspection::InspectionSnapshot;
use super::super::super::super::types::DebugSessionError;
use super::super::captures as graph;
use super::type_error;

pub(super) fn materialize(
    executable: &VerifiedExecutable,
    inspection: &InspectionSnapshot,
    function_id: FunctionId,
    function: &FunctionInfo,
    canonical: &str,
    frame_id: Option<u64>,
    limits: DebugEvaluationLimits,
) -> Result<Vec<Value>, DebugSessionError> {
    if function.debug.capture_sources.len() != usize::from(function.capture_count) {
        return Err(type_error(
            &format!("routine `{canonical}` has incomplete capture provenance"),
            "Assign a routine whose executable metadata records every capture source. Do not infer missing identities from names.",
        ));
    }
    for source in &function.debug.capture_sources {
        if source.kind != DebugCaptureKind::Value {
            return Err(type_error(
                &format!("routine `{canonical}` captures a mutable cell"),
                "Assign a named nested routine whose direct captures are immutable values, or copy a visible function binding.",
            ));
        }
    }
    let Some(owner) = function.debug.lexical_owner else {
        return Err(type_error(
            &format!("routine `{canonical}` has no lexical owner metadata"),
            "Assign a routine whose executable metadata records the exact enclosing function.",
        ));
    };
    let captures =
        inspection.read_value_captures(frame_id, owner, &function.debug.capture_sources)?;
    for (source, value) in function.debug.capture_sources.iter().zip(&captures) {
        super::super::super::validate_value(executable, source.ty, value, limits.max_depth)?;
    }
    let constructed = Value::function(function_id, canonical.to_string(), captures.clone(), false);
    let Value::Function(shared) = &constructed else {
        return Err(type_error(
            "capturing routine construction did not produce a function value",
            "Assign a named nested routine with immutable captures from its lexical-owner frame.",
        ));
    };
    graph::require_eligible(shared, limits.max_depth, limits.max_detached_values)?;
    Ok(captures)
}
