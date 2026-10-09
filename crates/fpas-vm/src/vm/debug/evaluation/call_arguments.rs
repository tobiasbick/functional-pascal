//! Written-order evaluation of named calls.
//! See `docs/pascal/language/functions/parameters.md#named-arguments`.

use fpas_bytecode::Value;

use super::execute::{EvaluationBudget, evaluate};
use super::model::{DebugCallTarget, DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

/// Evaluates values once, retaining names for the sandbox's parameter mapping.
pub(super) fn evaluate_call(
    target: DebugCallTarget,
    expressions: &[DebugExpression],
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut EvaluationBudget,
    resolve: &mut impl FnMut(&str) -> Result<Value, DebugSessionError>,
    invoke: &mut impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Value, DebugSessionError> {
    let named = expressions
        .iter()
        .any(|expression| matches!(expression, DebugExpression::NamedArgument { .. }));
    let mut names = Vec::new();
    let mut values = Vec::with_capacity(expressions.len());
    for expression in expressions {
        let value = match expression {
            DebugExpression::NamedArgument { name, value } => {
                names.push(name.clone());
                value.as_ref()
            }
            _ if named => {
                return Err(DebugSessionError {
                    kind: DebugErrorKind::EvaluationType,
                    message: "debug call mixes positional and named arguments".into(),
                    hint: "Use either all positional arguments or all named arguments.".into(),
                });
            }
            expression => expression,
        };
        values.push(evaluate(value, depth + 1, limits, budget, resolve, invoke)?);
    }
    let target = if named {
        DebugCallTarget::NamedArguments {
            target: Box::new(target),
            names,
        }
    } else {
        target
    };
    invoke(target, values)
}
