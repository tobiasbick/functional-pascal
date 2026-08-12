//! Stopped-state string character mutation transactions.

use fpas_bytecode::DebugType;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::{DebugAssignmentTarget, DebugStringMutationResult};

impl DebugSession {
    /// Replace one Unicode scalar below a mutable string debugger target.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, expression, type, index, resource, or availability error.
    /// Failure leaves live state and inspection handles unchanged.
    pub fn replace_string_character(
        &mut self,
        target: &DebugAssignmentTarget,
        index: &DebugExpression,
        value: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugStringMutationResult, DebugSessionError> {
        self.replace_string_character_with_limits(
            target,
            index,
            value,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Replace one Unicode scalar with explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::replace_string_character`].
    pub fn replace_string_character_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        index: &DebugExpression,
        value: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugStringMutationResult, DebugSessionError> {
        self.require_stopped("string.replace_character")?;
        let result = (|| {
            let prepared = self.prepare_sequence_mutation(
                assignment,
                &[index.clone(), value.clone()],
                frame_id,
                limits,
            )?;
            self.require_string_target(&prepared.target)?;
            let index = Self::sequence_index(&prepared.operands[0])?;
            let string_type = prepared.target.expected_type;
            super::super::super::mutation::validate_value(
                &self.executable,
                string_type,
                &prepared.operands[1],
                limits.max_depth,
            )?;
            let transformation = super::super::super::mutation::replace_string_character(
                prepared.sequence,
                index,
                prepared.operands[1].clone(),
            )?;
            let inspection = self
                .inspections
                .get(&prepared.task_id)
                .ok_or_else(|| unknown_task(prepared.task_id))?;
            let old_character =
                inspection.evaluation_summary(&transformation.old_character, limits)?;
            let new_character =
                inspection.evaluation_summary(&transformation.new_character, limits)?;
            let affected_index = transformation.index;
            let string = self.commit_sequence_value(
                prepared.task_id,
                &prepared.target,
                transformation.string,
                limits,
            )?;
            Ok(DebugStringMutationResult {
                string,
                index: affected_index,
                old_character,
                new_character,
            })
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    fn require_string_target(
        &self,
        target: &super::super::super::inspection::MutationTarget,
    ) -> Result<(), DebugSessionError> {
        match self
            .executable
            .executable()
            .debug_types
            .get(target.expected_type.get() as usize)
        {
            Some(DebugType::String) => Ok(()),
            _ => Err(DebugSessionError {
                kind: DebugErrorKind::VariablePathUnsupported,
                message: "debug string mutation target is not a string".to_string(),
                hint: "Select a mutable target whose complete value is a string.".to_string(),
            }),
        }
    }
}
