//! Bounded read-only expression evaluation over immutable stop snapshots.

mod execute;
mod model;

pub(super) use execute::evaluate_value;
pub use model::{
    DebugBinaryOperation, DebugEvaluateResult, DebugEvaluationLimits, DebugExpression,
    DebugUnaryOperation,
};
