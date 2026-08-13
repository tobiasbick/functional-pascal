//! Single and ordered multi-expression entry points over the bounded evaluator.

use fpas_bytecode::Value;

use super::execute::{EvaluationBudget, evaluate};
use super::model::{DebugCallTarget, DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::types::DebugSessionError;

pub(in crate::vm::debug) fn evaluate_value(
    expression: &DebugExpression,
    limits: DebugEvaluationLimits,
    mut resolve: impl FnMut(&str) -> Result<Value, DebugSessionError>,
    mut invoke: impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Value, DebugSessionError> {
    let mut budget = EvaluationBudget::new();
    evaluate(
        expression,
        0,
        limits,
        &mut budget,
        &mut resolve,
        &mut invoke,
    )
}

/// Evaluates multiple expressions in order under one shared resource budget.
pub(in crate::vm::debug) fn evaluate_values(
    expressions: &[DebugExpression],
    limits: DebugEvaluationLimits,
    mut resolve: impl FnMut(&str) -> Result<Value, DebugSessionError>,
    mut invoke: impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Vec<Value>, DebugSessionError> {
    let mut budget = EvaluationBudget::new();
    evaluate_batch(expressions, limits, &mut budget, &mut resolve, &mut invoke)
}

/// Evaluates a prefix, runs one validation checkpoint, then evaluates a suffix under one budget.
pub(in crate::vm::debug) fn evaluate_values_with_checkpoint<T>(
    prefix: &[DebugExpression],
    suffix: &[DebugExpression],
    limits: DebugEvaluationLimits,
    mut resolve: impl FnMut(&str) -> Result<Value, DebugSessionError>,
    mut invoke: impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
    checkpoint: impl FnOnce(&[Value]) -> Result<T, DebugSessionError>,
) -> Result<(T, Vec<Value>), DebugSessionError> {
    let mut budget = EvaluationBudget::new();
    let prefix = evaluate_batch(prefix, limits, &mut budget, &mut resolve, &mut invoke)?;
    let checkpoint = checkpoint(&prefix)?;
    let suffix = evaluate_batch(suffix, limits, &mut budget, &mut resolve, &mut invoke)?;
    Ok((checkpoint, suffix))
}

/// Evaluates a prefix, derives a suffix at a checkpoint, then evaluates it under one budget.
pub(in crate::vm::debug) fn evaluate_values_with_dynamic_suffix<T>(
    prefix: &[DebugExpression],
    limits: DebugEvaluationLimits,
    mut resolve: impl FnMut(&str) -> Result<Value, DebugSessionError>,
    mut invoke: impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
    checkpoint: impl FnOnce(&[Value]) -> Result<(T, Vec<DebugExpression>), DebugSessionError>,
) -> Result<(T, Vec<Value>), DebugSessionError> {
    let mut budget = EvaluationBudget::new();
    let prefix = evaluate_batch(prefix, limits, &mut budget, &mut resolve, &mut invoke)?;
    let (checkpoint, suffix) = checkpoint(&prefix)?;
    let suffix = evaluate_batch(&suffix, limits, &mut budget, &mut resolve, &mut invoke)?;
    Ok((checkpoint, suffix))
}

fn evaluate_batch(
    expressions: &[DebugExpression],
    limits: DebugEvaluationLimits,
    budget: &mut EvaluationBudget,
    resolve: &mut impl FnMut(&str) -> Result<Value, DebugSessionError>,
    invoke: &mut impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Vec<Value>, DebugSessionError> {
    expressions
        .iter()
        .map(|expression| evaluate(expression, 0, limits, budget, resolve, invoke))
        .collect()
}
