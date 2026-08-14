//! Task-selected stopped-state inspection and detached expression evaluation.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use super::*;
use crate::vm::debug::calls::LazyCallSandbox;
use crate::vm::debug::evaluation::{
    DebugEvaluateResult, DebugEvaluationLimits, DebugExpression, evaluate_value, evaluate_values,
    evaluate_values_with_checkpoint, evaluate_values_with_dynamic_suffix,
};
use crate::vm::debug::inspection::{DebugFrame, DebugScope, DebugVariable, Paginated};
use crate::vm::debug::types::{DebugTask, DebugTaskEvent};

impl DebugSession {
    /// Return a bounded page of logical frames for the current stop.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or inspection-limit error.
    pub fn stack(
        &self,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugFrame>, DebugSessionError> {
        self.require_inspectable("stack")?;
        self.current_inspection()?.stack(start, count)
    }

    /// Return a bounded page of logical frames for one stopped task.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, unknown-task, or inspection-limit error.
    pub fn stack_for_task(
        &mut self,
        task_id: u64,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugFrame>, DebugSessionError> {
        self.select_task(task_id)?;
        self.current_inspection()?.stack(start, count)
    }

    /// Select one inspectable stopped task as the default inspection context.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or unknown-task error.
    pub fn select_task(&mut self, task_id: u64) -> Result<(), DebugSessionError> {
        self.require_inspectable("task.select")?;
        self.select_inspection_task(task_id)
    }

    /// Return a bounded page of tasks captured by the all-stop session.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or inspection-limit error.
    pub fn tasks(
        &mut self,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugTask>, DebugSessionError> {
        self.require_inspectable("tasks")?;
        if count > self.inspection_limits.max_frames {
            return Err(DebugSessionError {
                kind: DebugErrorKind::InspectionLimit,
                message: format!(
                    "debug tasks page count {count} exceeds limit {}",
                    self.inspection_limits.max_frames
                ),
                hint: format!(
                    "Request at most {} tasks and use pagination.",
                    self.inspection_limits.max_frames
                ),
            });
        }
        let tasks = self.runtime.catalog();
        let total = tasks.len();
        Ok(Paginated {
            items: tasks.into_iter().skip(start).take(count).collect(),
            total,
        })
    }

    /// Drain task-lifecycle events accumulated since the previous call.
    #[must_use]
    pub fn take_task_events(&mut self) -> Vec<DebugTaskEvent> {
        self.runtime.take_events()
    }

    /// Return source scopes for one frame in the current stop snapshot.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or expired-frame error.
    pub fn scopes(&self, frame_id: u64) -> Result<Vec<DebugScope>, DebugSessionError> {
        self.require_inspectable("scopes")?;
        self.inspection_for_item(frame_id)?.scopes(frame_id)
    }

    /// Return one bounded page of variables or aggregate children.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, expired-reference, or inspection-limit error.
    pub fn variables(
        &mut self,
        reference: u64,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugVariable>, DebugSessionError> {
        self.require_inspectable("variables")?;
        self.inspection_for_item_mut(reference)?
            .variables(reference, start, count)
    }

    /// Evaluate one validated read-only expression against the current stop snapshot.
    ///
    /// A missing frame selects globals only. Supplied frame identifiers and returned aggregate
    /// handles are valid only for the current stop.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, frame, name, type, domain, unavailable-value, or limit error.
    pub fn evaluate(
        &mut self,
        expression: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.evaluate_with_limits(expression, frame_id, DebugEvaluationLimits::default())
    }

    /// Evaluate one validated read-only expression with explicit resource limits.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, frame, name, type, domain, unavailable-value, or limit error.
    pub fn evaluate_with_limits(
        &mut self,
        expression: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.require_inspectable("evaluate")?;
        let task_id = self.task_for_frame(frame_id)?;
        self.inspection_task_id = task_id;
        let result = self.evaluate_runtime_value(expression, frame_id, limits);
        let result = match result {
            Ok(value) => self
                .inspections
                .get_mut(&task_id)
                .ok_or_else(|| unknown_task(task_id))?
                .retain_evaluation_result(value, limits),
            Err(error) => Err(error),
        };
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    /// Evaluate one validated breakpoint condition as a strict Boolean value.
    ///
    /// # Errors
    ///
    /// Returns the normal evaluation failures or a type error for a non-Boolean result.
    pub fn evaluate_boolean(
        &self,
        expression: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<bool, DebugSessionError> {
        self.require_inspectable("evaluate.condition")?;
        let result = self
            .evaluate_runtime_value(expression, frame_id, DebugEvaluationLimits::default())
            .and_then(|value| match value {
                fpas_bytecode::Value::Boolean(value) => Ok(value),
                other => Err(DebugSessionError {
                    kind: DebugErrorKind::EvaluationType,
                    message: format!(
                        "debug breakpoint condition must be Boolean, got {}",
                        other.type_name()
                    ),
                    hint: "Use a comparison or another expression that returns Boolean."
                        .to_string(),
                }),
            });
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    /// Evaluates an expression to the underlying runtime value in its task context.
    pub(in crate::vm::debug) fn evaluate_runtime_value(
        &self,
        expression: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        inspection.validate_evaluation_frame(frame_id)?;
        let worker = self
            .runtime
            .worker(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let mut sandbox = LazyCallSandbox::new(
            Arc::clone(&self.executable),
            Arc::clone(&worker.layouts),
            Arc::clone(&worker.globals),
            limits,
            Arc::clone(&self.evaluation_cancelled),
        );
        evaluate_value(
            expression,
            limits,
            |name| inspection.resolve_evaluation_name(frame_id, name),
            |target, arguments| sandbox.invoke(target, arguments),
        )
    }

    /// Evaluates ordered expressions under one budget in one detached task context.
    pub(super) fn evaluate_runtime_values(
        &self,
        expressions: &[DebugExpression],
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<Vec<fpas_bytecode::Value>, DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        inspection.validate_evaluation_frame(frame_id)?;
        let worker = self
            .runtime
            .worker(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let mut sandbox = LazyCallSandbox::new(
            Arc::clone(&self.executable),
            Arc::clone(&worker.layouts),
            Arc::clone(&worker.globals),
            limits,
            Arc::clone(&self.evaluation_cancelled),
        );
        evaluate_values(
            expressions,
            limits,
            |name| inspection.resolve_evaluation_name(frame_id, name),
            |target, arguments| sandbox.invoke(target, arguments),
        )
    }

    /// Evaluates ordered expressions around one validation checkpoint under one shared budget.
    pub(super) fn evaluate_runtime_values_with_checkpoint<T>(
        &self,
        prefix: &[DebugExpression],
        suffix: &[DebugExpression],
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
        checkpoint: impl FnOnce(&[fpas_bytecode::Value]) -> Result<T, DebugSessionError>,
    ) -> Result<(T, Vec<fpas_bytecode::Value>), DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        inspection.validate_evaluation_frame(frame_id)?;
        let worker = self
            .runtime
            .worker(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let mut sandbox = LazyCallSandbox::new(
            Arc::clone(&self.executable),
            Arc::clone(&worker.layouts),
            Arc::clone(&worker.globals),
            limits,
            Arc::clone(&self.evaluation_cancelled),
        );
        evaluate_values_with_checkpoint(
            prefix,
            suffix,
            limits,
            |name| inspection.resolve_evaluation_name(frame_id, name),
            |target, arguments| sandbox.invoke(target, arguments),
            checkpoint,
        )
    }

    /// Evaluates selectors, derives ordered value expressions, and evaluates them under one budget.
    pub(super) fn evaluate_runtime_values_with_dynamic_suffix<T>(
        &self,
        prefix: &[DebugExpression],
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
        checkpoint: impl FnOnce(
            &[fpas_bytecode::Value],
        ) -> Result<(T, Vec<DebugExpression>), DebugSessionError>,
    ) -> Result<(T, Vec<fpas_bytecode::Value>), DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        inspection.validate_evaluation_frame(frame_id)?;
        let worker = self
            .runtime
            .worker(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let mut sandbox = LazyCallSandbox::new(
            Arc::clone(&self.executable),
            Arc::clone(&worker.layouts),
            Arc::clone(&worker.globals),
            limits,
            Arc::clone(&self.evaluation_cancelled),
        );
        evaluate_values_with_dynamic_suffix(
            prefix,
            limits,
            |name| inspection.resolve_evaluation_name(frame_id, name),
            |target, arguments| sandbox.invoke(target, arguments),
            checkpoint,
        )
    }
}
