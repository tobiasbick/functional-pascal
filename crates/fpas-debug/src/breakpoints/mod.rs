//! Protocol-neutral logical breakpoint conditions, hit counters, and logpoints.

mod hit_condition;
mod policy;

pub(crate) use policy::{BreakpointOutcome, BreakpointPolicy};
