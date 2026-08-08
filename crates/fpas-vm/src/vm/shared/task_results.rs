//! Retained task-result states shared by the scheduler and worker.

use fpas_bytecode::Value;

use crate::vm::VmError;

/// Non-blocking observation of one retained task result.
pub(crate) enum TaskResultPoll {
    /// The task is registered but has not completed.
    Pending,
    /// The task completed successfully with an unconsumed value.
    Available(Value),
    /// The task completed with its execution diagnostic.
    Failed(VmError),
    /// A previous wait consumed the successful result.
    Consumed,
    /// No retained task or consumed completion uses this identifier.
    Unknown,
}

/// Atomic observation of every retained task in a wait-all batch.
pub(crate) enum TaskBatchPoll {
    /// At least one registered task has not completed.
    Pending,
    /// Every task completed successfully or was already consumed.
    Complete,
    /// A task completed with its execution diagnostic.
    Failed(VmError),
    /// An identifier does not name a retained or consumed task.
    Unknown(u64),
}

/// Stored completion state for a retained spawned task.
pub(crate) enum TaskResultState {
    /// Registered before the task becomes visible to a worker.
    Pending,
    /// Completed successfully with the unit value.
    Unit,
    /// Completed successfully with a non-unit value.
    Value(Box<Value>),
    /// Completed with an execution diagnostic.
    Failed(Box<VmError>),
}
