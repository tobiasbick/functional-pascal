//! FPAS parser validation and lowering for read-only debugger expressions.

mod log_message;
mod parse;
mod target;
mod validate;

#[cfg(test)]
mod tests;

pub(crate) use log_message::{LogMessage, LogMessageLimits, LogSegment};
pub(crate) use parse::{EvaluationParseError, parse_debug_expression};
pub(crate) use target::parse_debug_assignment_target;
