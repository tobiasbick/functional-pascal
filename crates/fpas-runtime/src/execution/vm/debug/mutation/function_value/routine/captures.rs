//! Materialize Value, Cell, and EnclosingCell captures from the selected owner frame.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use crate::{DebugCaptureKind, FunctionId, FunctionInfo, Value, VerifiedExecutable};

use super::super::super::super::evaluation::DebugEvaluationLimits;
use super::super::super::super::inspection::{InspectionSnapshot, MutationTarget};
use super::super::super::super::types::DebugSessionError;
use super::super::captures as graph;
use super::type_error;

#[expect(
    clippy::too_many_arguments,
    reason = "capture materialization keeps executable, owner-frame, task, and destination proofs together"
)]
pub(super) fn materialize(
    executable: &VerifiedExecutable,
    inspection: &InspectionSnapshot,
    function_id: FunctionId,
    function: &FunctionInfo,
    canonical: &str,
    frame_id: Option<u64>,
    _task_id: u64,
    destination: Option<&MutationTarget>,
    limits: DebugEvaluationLimits,
) -> Result<Value, DebugSessionError> {
    if function.debug.capture_sources.len() != usize::from(function.capture_count) {
        return Err(type_error(
            &format!("routine `{canonical}` has incomplete capture provenance"),
            "Assign a routine whose executable metadata records every capture source. Do not infer missing identities from names.",
        ));
    }
    let Some(owner) = function.debug.lexical_owner else {
        return Err(type_error(
            &format!("routine `{canonical}` has no lexical owner metadata"),
            "Assign a routine whose executable metadata records the exact enclosing function.",
        ));
    };
    let captures = inspection.read_captures(frame_id, owner, &function.debug.capture_sources)?;
    let mut task_bound = false;
    for (source, value) in function.debug.capture_sources.iter().zip(&captures) {
        match source.kind {
            DebugCaptureKind::Value => {
                super::super::super::validate_value(
                    executable,
                    source.ty,
                    value,
                    limits.max_depth,
                )?;
            }
            DebugCaptureKind::Cell | DebugCaptureKind::EnclosingCell => {
                task_bound = true;
            }
        }
    }
    let classification = crate::classify_values(&captures)
        .map_err(|_| type_error("capture graph is unavailable", "Retry at a stable stop."))?;
    classification
        .validate_domain(&inspection.domain)
        .map_err(|_| {
            type_error(
                "capture graph contains foreign local values",
                "Select the owning stopped task.",
            )
        })?;
    task_bound |= !classification.is_transferable();
    let target = executable.function_target(function_id).ok_or_else(|| {
        type_error(
            "capture target is outside the verified executable",
            "Rebuild the executable.",
        )
    })?;
    let constructed = if task_bound {
        super::super::destination::require_frame_register(
            destination,
            frame_id,
            inspection.generation(),
        )?;
        Value::task_owned_function(target, captures, inspection.domain.id().clone())
    } else {
        Value::function(target, captures)
    };
    let Value::Function(shared) = &constructed else {
        return Err(type_error(
            "capturing routine construction did not produce a function value",
            "Assign a named nested routine from its lexical-owner frame.",
        ));
    };
    if task_bound {
        graph::require_task_owned(shared, limits.max_depth, limits.max_detached_values)?;
    } else {
        graph::require_eligible(shared, limits.max_depth, limits.max_detached_values)?;
    }
    Ok(constructed)
}
