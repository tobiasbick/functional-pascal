//! Portable type graphs retained for debugger-side value validation.

use crate::{DebugTypeId, EnumTypeId, RecordTypeId};

/// Runtime-independent FPAS type used by debugger inspection and mutation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DebugType {
    /// Procedure result type.
    Unit,
    /// Boolean value.
    Boolean,
    /// Signed integer value.
    Integer,
    /// IEEE-754 real value.
    Real,
    /// UTF-8 string value.
    String,
    /// Dynamically checked value.
    Dynamic,
    /// Ordered array with one element type.
    Array(DebugTypeId),
    /// Ordered dictionary with fixed key and value types.
    Dictionary {
        /// Key type.
        key: DebugTypeId,
        /// Stored value type.
        value: DebugTypeId,
    },
    /// Result with success and error payload types.
    Result {
        /// Success payload type.
        ok: DebugTypeId,
        /// Error payload type.
        error: DebugTypeId,
    },
    /// Optional value.
    Option(DebugTypeId),
    /// First-class callable signature.
    Function {
        /// Ordered parameter types.
        parameters: Vec<DebugTypeId>,
        /// Result type.
        result: DebugTypeId,
    },
    /// Record value with an executable layout.
    Record(RecordTypeId),
    /// Enum value with an executable layout.
    Enum(EnumTypeId),
    /// Shared mutable capture cell.
    Cell(DebugTypeId),
    /// Task handle result type.
    Task(DebugTypeId),
    /// Typed channel handle element type.
    Channel(DebugTypeId),
}
