//! Stopped-state array structure mutation transactions.

use fpas_bytecode::DebugType;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::{DebugArrayMutationResult, DebugAssignmentTarget};

impl DebugSession {
    /// Insert one array element below a mutable debugger target.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, expression, type, index, resource, or availability error.
    /// Failure leaves live state and inspection handles unchanged.
    pub fn insert_array_element(
        &mut self,
        target: &DebugAssignmentTarget,
        index: &DebugExpression,
        value: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugArrayMutationResult, DebugSessionError> {
        self.insert_array_element_with_limits(
            target,
            index,
            value,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Insert one array element with explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::insert_array_element`].
    pub fn insert_array_element_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        index: &DebugExpression,
        value: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugArrayMutationResult, DebugSessionError> {
        self.require_stopped("array.insert")?;
        let result = (|| {
            let prepared = self.prepare_sequence_mutation(
                assignment,
                &[index.clone(), value.clone()],
                frame_id,
                limits,
            )?;
            let element_type = self.array_element_type(&prepared.target)?;
            let index = Self::sequence_index(&prepared.operands[0])?;
            super::super::super::mutation::validate_value(
                &self.executable,
                element_type,
                &prepared.operands[1],
                limits.max_depth,
            )?;
            let transformation = super::super::super::mutation::insert_array(
                prepared.sequence,
                index,
                prepared.operands[1].clone(),
            )?;
            let affected_index = transformation.index;
            let array = self.commit_sequence_value(
                prepared.task_id,
                &prepared.target,
                transformation.array,
                limits,
            )?;
            Ok(DebugArrayMutationResult {
                array,
                index: affected_index,
                removed: None,
            })
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    /// Remove one array element below a mutable debugger target.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, expression, index, resource, or availability error.
    /// Failure leaves live state and inspection handles unchanged.
    pub fn remove_array_element(
        &mut self,
        target: &DebugAssignmentTarget,
        index: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugArrayMutationResult, DebugSessionError> {
        self.remove_array_element_with_limits(
            target,
            index,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Remove one array element with explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::remove_array_element`].
    pub fn remove_array_element_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        index: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugArrayMutationResult, DebugSessionError> {
        self.require_stopped("array.remove")?;
        let result = (|| {
            let prepared = self.prepare_sequence_mutation(
                assignment,
                std::slice::from_ref(index),
                frame_id,
                limits,
            )?;
            self.array_element_type(&prepared.target)?;
            let index = Self::sequence_index(&prepared.operands[0])?;
            let transformation =
                super::super::super::mutation::remove_array(prepared.sequence, index)?;
            let inspection = self
                .inspections
                .get(&prepared.task_id)
                .ok_or_else(|| unknown_task(prepared.task_id))?;
            let removed = transformation
                .removed
                .as_ref()
                .map(|value| inspection.evaluation_summary(value, limits))
                .transpose()?;
            let affected_index = transformation.index;
            let array = self.commit_sequence_value(
                prepared.task_id,
                &prepared.target,
                transformation.array,
                limits,
            )?;
            Ok(DebugArrayMutationResult {
                array,
                index: affected_index,
                removed,
            })
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    fn array_element_type(
        &self,
        target: &super::super::super::inspection::MutationTarget,
    ) -> Result<fpas_bytecode::DebugTypeId, DebugSessionError> {
        match self
            .executable
            .executable()
            .debug_types
            .get(target.expected_type.get() as usize)
        {
            Some(DebugType::Array(element)) => Ok(*element),
            _ => Err(DebugSessionError {
                kind: DebugErrorKind::VariablePathUnsupported,
                message: "debug array mutation target is not an array".to_string(),
                hint: "Select a mutable target whose complete value is an array.".to_string(),
            }),
        }
    }
}
