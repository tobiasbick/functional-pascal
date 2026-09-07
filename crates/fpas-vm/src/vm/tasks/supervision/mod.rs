//! Bounded worker retries that retain one task identity and group membership.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

mod execution;
#[cfg(test)]
mod lifetime_tests;
mod policy;
#[cfg(test)]
mod tests;

use fpas_bytecode::SharedFunction;
use std::time::Instant;

pub(in crate::vm) use policy::RetryPolicy;

enum Deadline {
    Realtime(Instant),
    Debug(u64),
}

/// Owned retry inputs travel with a task across scheduler suspension.
pub(in crate::vm) struct SupervisedTask {
    function: SharedFunction,
    token: u64,
    policy: RetryPolicy,
    admission_pending: bool,
    deadline: Option<Deadline>,
}

impl SupervisedTask {
    /// Retain immutable callable inputs only after group admission succeeds.
    pub(in crate::vm) fn new(function: SharedFunction, token: u64, policy: RetryPolicy) -> Self {
        Self {
            function,
            token,
            policy,
            admission_pending: true,
            deadline: None,
        }
    }
}
