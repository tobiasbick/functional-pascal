//! Bounded read-only expression evaluation over immutable stop snapshots.

mod batch;
mod execute;
mod model;
mod qualified;

pub(super) use batch::{
    evaluate_value, evaluate_values, evaluate_values_with_checkpoint,
    evaluate_values_with_dynamic_suffix,
};
pub(super) use model::DebugCallTarget;
pub use model::{
    DebugBinaryOperation, DebugEvaluateResult, DebugEvaluationLimits, DebugExpression,
    DebugUnaryOperation,
};
