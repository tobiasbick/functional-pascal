//! The primary failure of a task scope and the further failures attached to it.
//!
//! Documentation: `docs/pascal/language/concurrency/task-scopes.md#failures`.

use fpas_diagnostics::codes::{RUNTIME_TASK_CANCELLED, RUNTIME_VM_SHUTDOWN};

use crate::vm::VmError;

/// One failed child, in observation order and child start order.
#[derive(Clone)]
pub(in crate::vm) struct ScopeFailure {
    /// Global observation order; the smallest value is the primary failure.
    pub(in crate::vm) sequence: u64,
    /// Position of the child among the scope's started children.
    pub(in crate::vm) start_order: usize,
    /// Original diagnostic of the child.
    pub(in crate::vm) error: VmError,
}

/// Whether a task ended because it was cancelled rather than because it failed.
pub(in crate::vm) fn is_cancellation(error: &VmError) -> bool {
    matches!(error.code, RUNTIME_TASK_CANCELLED | RUNTIME_VM_SHUTDOWN)
}

/// Select the first observed failure and attach the others in child start order.
///
/// `body` is the error that ended the scope body, with its observation order. It joins the
/// failures unless it is a cancellation or repeats a child failure observed through `Wait`.
pub(in crate::vm) fn scope_outcome(
    mut failures: Vec<ScopeFailure>,
    body: Option<(u64, VmError)>,
) -> Option<(u64, VmError)> {
    let mut body_failure = None;
    if let Some((sequence, error)) = body {
        if is_cancellation(&error) {
            if failures.is_empty() {
                return Some((sequence, error));
            }
        } else if !failures.iter().any(|failure| failure.sequence == sequence) {
            body_failure = Some((sequence, error));
        }
    }
    failures.sort_by_key(|failure| failure.start_order);
    let child_primary = failures
        .iter()
        .enumerate()
        .min_by_key(|(_, failure)| failure.sequence)
        .map(|(index, failure)| (index, failure.sequence));
    let (mut primary, further) = match (body_failure, child_primary) {
        (Some((body_sequence, body_error)), Some((index, child_sequence)))
            if child_sequence < body_sequence =>
        {
            let primary = failures.remove(index);
            let mut further = vec![body_error];
            further.extend(failures.into_iter().map(|failure| failure.error));
            ((primary.sequence, primary.error), further)
        }
        (Some((body_sequence, body_error)), _) => (
            (body_sequence, body_error),
            failures.into_iter().map(|failure| failure.error).collect(),
        ),
        (None, Some((index, _))) => {
            let primary = failures.remove(index);
            (
                (primary.sequence, primary.error),
                failures.into_iter().map(|failure| failure.error).collect(),
            )
        }
        (None, None) => return None,
    };
    attach_further_failures(&mut primary.1, &further);
    Some(primary)
}

fn attach_further_failures(primary: &mut VmError, further: &[VmError]) {
    if further.is_empty() {
        return;
    }
    let details = further
        .iter()
        .map(|error| match error.span {
            Some(span) => format!(
                "FP{} at {}:{}: {}",
                error.code.value(),
                span.line(),
                span.column(),
                error.message
            ),
            None => format!("FP{}: {}", error.code.value(), error.message),
        })
        .collect::<Vec<_>>()
        .join("; ");
    let help = primary.help.get_or_insert_default();
    if !help.is_empty() {
        help.push(' ');
    }
    help.push_str(&format!(
        "Further failures in this task scope ({}): {details}.",
        further.len()
    ));
}

#[cfg(test)]
mod tests;
