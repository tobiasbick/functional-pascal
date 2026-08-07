//! Runtime-independent constant table values.

use crate::{FunctionId, StringId};

/// Persistent register-bytecode constant with deterministic bit identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Constant {
    /// Signed 64-bit integer.
    Integer(i64),
    /// Exact IEEE-754 bit pattern, preserving NaN payloads and signed zero.
    Real(u64),
    /// Boolean value.
    Boolean(bool),
    /// Validated UTF-8 string table reference.
    String(StringId),
    /// Procedure result value.
    Unit,
    /// Non-capturing function value.
    Function {
        /// Numeric function table reference.
        function: FunctionId,
        /// Whether invocation is restricted to the creating task.
        task_bound: bool,
    },
}
