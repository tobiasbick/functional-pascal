//! Bounded source and function breakpoint binding.

mod function;
mod source;

pub use function::{BoundFunctionBreakpoint, DebugBreakpointLimits, FunctionBreakpoint};
pub use source::{BoundBreakpoint, SourceBreakpoint};

pub(super) use function::bind as bind_function;
pub(super) use source::{bind as bind_source, point_at, source_location};
