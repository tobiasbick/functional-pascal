//! Breakpoint-action assignment through durable location identities.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluateResult, DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::inspection::{MutationRoot, MutationTarget};
use crate::vm::debug::location::DebugDataLocationIdentity;

impl DebugSession {
    /// Assign one durable location using the stopped-state mutation transaction.
    ///
    /// Only executable-global identities are accepted. The replacement is
    /// prepared and validated before a single commit; inspection snapshots
    /// refresh once on success and stay unchanged on failure.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, identity, type, or evaluation error without
    /// mutating live storage when validation fails.
    pub fn assign_data_location(
        &mut self,
        identity: DebugDataLocationIdentity,
        expression: &DebugExpression,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.assign_data_location_in_frame(identity, expression, None)
    }

    /// Assign one durable location, evaluating `expression` in `frame_id`.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::assign_data_location`].
    pub fn assign_data_location_in_frame(
        &mut self,
        identity: DebugDataLocationIdentity,
        expression: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.require_stopped("location.assign")?;
        let limits = DebugEvaluationLimits::default();
        let result = (|| {
            let task_id = self.task_for_frame(frame_id)?;
            let target = self.mutation_target_for_identity(identity, task_id)?;
            let mut evaluation_target = target.clone();
            evaluation_target.frame_id = frame_id;
            let replacement = self.evaluate_replacement_for_target(
                task_id,
                &evaluation_target,
                expression,
                limits,
            )?;
            self.commit_mutation(task_id, &target, replacement, limits)
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    fn mutation_target_for_identity(
        &self,
        identity: DebugDataLocationIdentity,
        task_id: u64,
    ) -> Result<MutationTarget, DebugSessionError> {
        let DebugDataLocationIdentity::Global { index } = identity else {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariablePathUnsupported,
                message: "breakpoint assign requires an executable-global location identity"
                    .to_string(),
                hint: "Use `location.describe` on a global; frame registers and capture cells are not assignable from breakpoint actions.".to_string(),
            });
        };
        let index = usize::try_from(index).map_err(|_| unknown_global(index))?;
        let global = self
            .executable
            .executable()
            .globals
            .get(index)
            .ok_or_else(|| unknown_global(index as u64))?;
        if !global.mutable {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableNotMutable,
                message: format!("debug variable target global slot {index} is not mutable"),
                hint: "Select a source-declared mutable global.".to_string(),
            });
        }
        let generation = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .generation();
        let initialized = self.runtime.worker(task_id).is_some_and(|worker| {
            let globals = worker
                .globals
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            globals.get(index).cloned().flatten().is_some()
        });
        Ok(MutationTarget {
            root: MutationRoot::Global(index),
            path: Vec::new(),
            expected_type: global.ty,
            generation,
            frame_id: None,
            initialized,
            initializer: None,
        })
    }
}

fn unknown_global(index: u64) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!("debug location identity global slot {index} is unknown"),
        hint: "Use an identity returned by `location.describe` for a current global.".to_string(),
    }
}
