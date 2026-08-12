//! Shared preparation and commit support for stopped-state sequence mutations.

mod array;
mod string;

use fpas_bytecode::Value;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::{DebugAssignmentSelector, DebugAssignmentTarget};

impl DebugSession {
    fn prepare_sequence_mutation(
        &self,
        assignment: &DebugAssignmentTarget,
        operation_expressions: &[DebugExpression],
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<PreparedSequenceMutation, DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let (target, current) = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .resolve_named_mutation_target(frame_id, &assignment.root)?;
        let current = current.ok_or_else(|| uninitialized_sequence(&assignment.root))?;
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
        let (target, sequence) = super::super::mutation::target_with_value(
            self.executable.executable(),
            assignment,
            target,
            current,
            &values,
        )?;
        if operands.len() != operation_expressions.len() {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableUnavailable,
                message: "debug sequence mutation input is unavailable".to_string(),
                hint: "Retry with complete index and replacement expressions.".to_string(),
            });
        }
        Ok(PreparedSequenceMutation {
            task_id,
            target,
            sequence,
            operands,
        })
    }

    fn sequence_index(value: &Value) -> Result<i64, DebugSessionError> {
        match value {
            Value::Integer(index) => Ok(*index),
            _ => Err(DebugSessionError {
                kind: DebugErrorKind::VariableValueType,
                message: "debug sequence index expression must produce an Integer".to_string(),
                hint: "Use a zero-based Integer expression for the sequence index.".to_string(),
            }),
        }
    }

    fn commit_sequence_value(
        &mut self,
        task_id: u64,
        target: &super::super::inspection::MutationTarget,
        replacement: Value,
        limits: DebugEvaluationLimits,
    ) -> Result<crate::vm::debug::evaluation::DebugEvaluateResult, DebugSessionError> {
        super::super::mutation::validate_replacement(
            &self.executable,
            target,
            &replacement,
            limits.max_depth,
        )?;
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        inspection.evaluation_summary(&replacement, limits)?;
        let generation = inspection.generation();
        let worker = self
            .runtime
            .worker_mut(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let committed = super::super::mutation::commit(worker, generation, target, replacement)?;
        self.invalidate_inspection();
        self.refresh_inspection();
        self.inspection_task_id = task_id;
        self.inspections
            .get_mut(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .retain_evaluation_result(committed, limits)
    }
}

struct PreparedSequenceMutation {
    task_id: u64,
    target: super::super::inspection::MutationTarget,
    sequence: Value,
    operands: Vec<Value>,
}

fn uninitialized_sequence(name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: format!(
            "debug variable target `{name}` has no writable descendants before initialization"
        ),
        hint:
            "Initialize the complete binding before inserting, removing, or replacing characters."
                .to_string(),
    }
}
