//! Protocol-neutral logical breakpoint conditions, hit counters, and logpoints.

mod assign;
mod hit_condition;
mod policy;
mod runtime_failure;

pub(crate) use assign::BreakpointAssign;
pub(crate) use policy::{BreakpointOutcome, BreakpointPolicy};
pub(crate) use runtime_failure::{MAX_RUNTIME_FAILURE_FILTERS, RuntimeFailurePolicy};
