//! Stopped-state dictionary structure mutation transactions.

use fpas_bytecode::{DebugType, DebugTypeId, Value};

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::{
    DebugAssignmentSelector, DebugAssignmentTarget, DebugDictionaryMutationResult,
    DictionaryTransformation,
};

impl DebugSession {
    /// Insert one missing dictionary entry below a mutable debugger target.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, expression, type, collision, resource, or availability
    /// error. Failure leaves live state and inspection handles unchanged.
    pub fn insert_dictionary_entry(
        &mut self,
        target: &DebugAssignmentTarget,
        key: &DebugExpression,
        value: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugDictionaryMutationResult, DebugSessionError> {
        self.insert_dictionary_entry_with_limits(
            target,
            key,
            value,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Insert one missing dictionary entry with explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::insert_dictionary_entry`].
    pub fn insert_dictionary_entry_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        key: &DebugExpression,
        value: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugDictionaryMutationResult, DebugSessionError> {
        self.require_stopped("dictionary.insert")?;
        let result = (|| {
            let prepared = self.prepare_dictionary_mutation(
                assignment,
                &[key.clone(), value.clone()],
                frame_id,
                limits,
            )?;
            let (key_type, value_type) = self.dictionary_types(&prepared.target)?;
            super::super::mutation::validate_value(
                &self.executable,
                key_type,
                &prepared.operands[0],
                limits.max_depth,
            )?;
            super::super::mutation::validate_value(
                &self.executable,
                value_type,
                &prepared.operands[1],
                limits.max_depth,
            )?;
            let transformation = super::super::mutation::insert(
                prepared.dictionary,
                prepared.operands[0].clone(),
                prepared.operands[1].clone(),
            )?;
            self.commit_dictionary_mutation(
                prepared.task_id,
                &prepared.target,
                transformation,
                limits,
            )
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    /// Remove one existing dictionary entry below a mutable debugger target.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, expression, type, missing-key, resource, or availability
    /// error. Failure leaves live state and inspection handles unchanged.
    pub fn remove_dictionary_entry(
        &mut self,
        target: &DebugAssignmentTarget,
        key: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugDictionaryMutationResult, DebugSessionError> {
        self.remove_dictionary_entry_with_limits(
            target,
            key,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Remove one existing dictionary entry with explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::remove_dictionary_entry`].
    pub fn remove_dictionary_entry_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        key: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugDictionaryMutationResult, DebugSessionError> {
        self.require_stopped("dictionary.remove")?;
        let result = (|| {
            let prepared = self.prepare_dictionary_mutation(
                assignment,
                std::slice::from_ref(key),
                frame_id,
                limits,
            )?;
            let (key_type, _) = self.dictionary_types(&prepared.target)?;
            super::super::mutation::validate_value(
                &self.executable,
                key_type,
                &prepared.operands[0],
                limits.max_depth,
            )?;
            let transformation =
                super::super::mutation::remove(prepared.dictionary, &prepared.operands[0])?;
            self.commit_dictionary_mutation(
                prepared.task_id,
                &prepared.target,
                transformation,
                limits,
            )
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    /// Replace one existing dictionary key with one missing key.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, expression, type, collision, missing-key, no-op,
    /// resource, or availability error. Failure leaves live state and handles unchanged.
    pub fn replace_dictionary_key(
        &mut self,
        target: &DebugAssignmentTarget,
        old_key: &DebugExpression,
        new_key: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugDictionaryMutationResult, DebugSessionError> {
        self.replace_dictionary_key_with_limits(
            target,
            old_key,
            new_key,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Replace one dictionary key with explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::replace_dictionary_key`].
    pub fn replace_dictionary_key_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        old_key: &DebugExpression,
        new_key: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugDictionaryMutationResult, DebugSessionError> {
        self.require_stopped("dictionary.replace_key")?;
        let result = (|| {
            let prepared = self.prepare_dictionary_mutation(
                assignment,
                &[old_key.clone(), new_key.clone()],
                frame_id,
                limits,
            )?;
            let (key_type, _) = self.dictionary_types(&prepared.target)?;
            for key in &prepared.operands {
                super::super::mutation::validate_value(
                    &self.executable,
                    key_type,
                    key,
                    limits.max_depth,
                )?;
            }
            let transformation = super::super::mutation::replace_key(
                prepared.dictionary,
                &prepared.operands[0],
                prepared.operands[1].clone(),
            )?;
            self.commit_dictionary_mutation(
                prepared.task_id,
                &prepared.target,
                transformation,
                limits,
            )
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    fn prepare_dictionary_mutation(
        &self,
        assignment: &DebugAssignmentTarget,
        operation_expressions: &[DebugExpression],
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<PreparedDictionaryMutation, DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let (target, current) = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .resolve_named_mutation_target(frame_id, &assignment.root)?;
        let selector_count = assignment
            .selectors
            .iter()
            .filter(|selector| matches!(selector, DebugAssignmentSelector::Index(_)))
            .count();
        let mut expressions = assignment
            .selectors
            .iter()
            .filter_map(|selector| match selector {
                DebugAssignmentSelector::Field(_) => None,
                DebugAssignmentSelector::Index(expression) => Some(expression.clone()),
            })
            .collect::<Vec<_>>();
        expressions.extend_from_slice(operation_expressions);
        let mut values = self.evaluate_runtime_values(&expressions, frame_id, limits)?;
        let operands = values.split_off(selector_count);
        let (target, dictionary) = super::super::mutation::target_with_value(
            self.executable.executable(),
            assignment,
            target,
            current,
            &values,
        )?;
        if operands.len() != operation_expressions.len() {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableUnavailable,
                message: "debug dictionary mutation input is unavailable".to_string(),
                hint: "Retry with complete key and value expressions.".to_string(),
            });
        }
        Ok(PreparedDictionaryMutation {
            task_id,
            target,
            dictionary,
            operands,
        })
    }

    fn dictionary_types(
        &self,
        target: &super::super::inspection::MutationTarget,
    ) -> Result<(DebugTypeId, DebugTypeId), DebugSessionError> {
        match self
            .executable
            .executable()
            .debug_types
            .get(target.expected_type.get() as usize)
        {
            Some(DebugType::Dictionary { key, value }) => Ok((*key, *value)),
            _ => Err(DebugSessionError {
                kind: DebugErrorKind::VariablePathUnsupported,
                message: "debug dictionary mutation target is not a dictionary".to_string(),
                hint: "Select a mutable target whose complete value is `dict of K to V`."
                    .to_string(),
            }),
        }
    }

    fn commit_dictionary_mutation(
        &mut self,
        task_id: u64,
        target: &super::super::inspection::MutationTarget,
        transformation: DictionaryTransformation,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugDictionaryMutationResult, DebugSessionError> {
        super::super::mutation::validate_replacement(
            &self.executable,
            target,
            &transformation.dictionary,
            limits.max_depth,
        )?;
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let removed = transformation
            .removed
            .as_ref()
            .map(|value| inspection.evaluation_summary(value, limits))
            .transpose()?;
        let old_key = transformation
            .old_key
            .as_ref()
            .map(|value| inspection.evaluation_summary(value, limits))
            .transpose()?;
        let new_key = transformation
            .new_key
            .as_ref()
            .map(|value| inspection.evaluation_summary(value, limits))
            .transpose()?;
        inspection.evaluation_summary(&transformation.dictionary, limits)?;

        let generation = inspection.generation();
        let worker = self
            .runtime
            .worker_mut(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let committed =
            super::super::mutation::commit(worker, generation, target, transformation.dictionary)?;
        self.invalidate_inspection();
        self.refresh_inspection();
        self.inspection_task_id = task_id;
        let dictionary = self
            .inspections
            .get_mut(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .retain_evaluation_result(committed, limits)?;
        Ok(DebugDictionaryMutationResult {
            dictionary,
            removed,
            old_key,
            new_key,
        })
    }
}

struct PreparedDictionaryMutation {
    task_id: u64,
    target: super::super::inspection::MutationTarget,
    dictionary: Value,
    operands: Vec<Value>,
}
