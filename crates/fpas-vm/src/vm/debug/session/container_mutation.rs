//! Shared preparation and commit for stopped-state container structure mutations.

use fpas_bytecode::Value;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluateResult, DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::inspection::MutationTarget;
use crate::vm::debug::mutation::{DebugAssignmentSelector, DebugAssignmentTarget};

/// Container family edited by a structure mutation.
#[derive(Clone, Copy)]
pub(super) enum ContainerKind {
    /// Array or string.
    Sequence,
    /// Dictionary.
    Dictionary,
}

impl ContainerKind {
    fn name(self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::Dictionary => "dictionary",
        }
    }

    fn incomplete_input_hint(self) -> &'static str {
        match self {
            Self::Sequence => "Retry with complete index and replacement expressions.",
            Self::Dictionary => "Retry with complete key and value expressions.",
        }
    }

    fn uninitialized_hint(self) -> &'static str {
        match self {
            Self::Sequence => {
                "Initialize the complete binding before inserting, removing, or replacing characters."
            }
            Self::Dictionary => {
                "Initialize the complete binding before inserting, removing, or replacing entries."
            }
        }
    }
}

/// Resolved container target and evaluated operation operands.
pub(super) struct PreparedContainerMutation {
    pub(super) task_id: u64,
    pub(super) target: MutationTarget,
    /// Current container value.
    pub(super) container: Value,
    /// Operation operands in the order of the operation expressions.
    pub(super) operands: Vec<Value>,
}

impl DebugSession {
    /// Resolve `assignment` to its current container value and evaluate its index
    /// selectors together with `operation_expressions`.
    pub(super) fn prepare_container_mutation(
        &self,
        kind: ContainerKind,
        assignment: &DebugAssignmentTarget,
        operation_expressions: &[DebugExpression],
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<PreparedContainerMutation, DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let (target, current) = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .resolve_named_mutation_target(frame_id, &assignment.root)?;
        let current = current.ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::VariablePathUnsupported,
            message: format!(
                "debug variable target `{}` has no writable descendants before initialization",
                assignment.root
            ),
            hint: kind.uninitialized_hint().to_string(),
        })?;
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
        let (target, container) = super::super::mutation::target_with_value(
            self.executable.executable(),
            assignment,
            target,
            current,
            &values,
        )?;
        if operands.len() != operation_expressions.len() {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableUnavailable,
                message: format!("debug {} mutation input is unavailable", kind.name()),
                hint: kind.incomplete_input_hint().to_string(),
            });
        }
        Ok(PreparedContainerMutation {
            task_id,
            target,
            container,
            operands,
        })
    }

    /// Validate and commit a replacement container, then retain its rendered result.
    ///
    /// `summarize` renders extra operation details before the replacement is summarized
    /// and committed.
    pub(super) fn commit_container_value<T>(
        &mut self,
        task_id: u64,
        target: &MutationTarget,
        replacement: Value,
        limits: DebugEvaluationLimits,
        summarize: impl FnOnce(&InspectionSnapshot) -> Result<T, DebugSessionError>,
    ) -> Result<(DebugEvaluateResult, T), DebugSessionError> {
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
        let details = summarize(inspection)?;
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
        let result = self
            .inspections
            .get_mut(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .retain_evaluation_result(committed, limits)?;
        Ok((result, details))
    }
}
