//! Bounded worker retries that retain one task identity and group membership.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

#[cfg(test)]
mod case_tests;
mod execution;
#[cfg(test)]
mod lifetime_tests;
mod policy;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod transfer_tests;

use crate::SharedFunction;
use std::time::Instant;

pub(in crate::execution::vm) use policy::RetryPolicy;

enum Deadline {
    Realtime(Instant),
    Debug(u64),
}

/// Owned retry inputs travel with a task across scheduler suspension.
pub(in crate::execution::vm) struct SupervisedTask {
    function: SharedFunction,
    token: u64,
    policy: RetryPolicy,
    admission_pending: bool,
    deadline: Option<Deadline>,
}

impl SupervisedTask {
    /// Whether retry admission or backoff is still pending before the attempt starts.
    pub(in crate::execution::vm) fn is_admission_pending(&self) -> bool {
        self.admission_pending
    }

    /// Retain immutable callable inputs only after group admission succeeds.
    pub(in crate::execution::vm) fn new(
        function: SharedFunction,
        token: u64,
        policy: RetryPolicy,
    ) -> Self {
        Self {
            function,
            token,
            policy,
            admission_pending: true,
            deadline: None,
        }
    }
}

impl crate::TraceRoots for SupervisedTask {
    fn trace_roots(
        &self,
        visit: &mut dyn FnMut(&crate::Value),
    ) -> Result<(), crate::OwnershipError> {
        visit(&crate::Value::Function(self.function.clone()));
        Ok(())
    }
}
