//! Bounded read-only expression evaluation over immutable stop snapshots.

mod execute;
mod model;
mod qualified;

pub(super) use execute::{evaluate_value, evaluate_values, evaluate_values_with_checkpoint};
pub(super) use model::DebugCallTarget;
pub use model::{
    DebugBinaryOperation, DebugEvaluateResult, DebugEvaluationLimits, DebugExpression,
    DebugUnaryOperation,
};
