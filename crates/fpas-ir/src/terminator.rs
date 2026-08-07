//! Explicit control-flow terminators and block arguments.

use crate::{BlockId, ValueId};

/// A target block together with its merge-value arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockTarget {
    /// Destination block in the current function.
    pub block: BlockId,
    /// Values supplied to the destination block parameters.
    pub arguments: Vec<ValueId>,
}

/// The sole control-flow exit of a basic block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    /// Selects one of two successor blocks using a boolean condition.
    Branch {
        /// Boolean condition value.
        condition: ValueId,
        /// Successor when the condition is true.
        then_target: BlockTarget,
        /// Successor when the condition is false.
        else_target: BlockTarget,
    },
    /// Transfers unconditionally to one successor block.
    Jump(BlockTarget),
    /// Returns an optional Unit or typed function result.
    Return(Option<ValueId>),
    /// Terminates evaluation with a value used to construct a diagnostic.
    Panic(ValueId),
}

impl Terminator {
    /// Returns successor block targets in deterministic source order.
    #[must_use]
    pub fn targets(&self) -> Vec<&BlockTarget> {
        match self {
            Self::Branch {
                then_target,
                else_target,
                ..
            } => vec![then_target, else_target],
            Self::Jump(target) => vec![target],
            Self::Return(_) | Self::Panic(_) => Vec::new(),
        }
    }
}
