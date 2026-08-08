//! Runtime-owned construction of typed record and enum values returned by intrinsics.

use fpas_bytecode::{SourceLocation, Value};

use crate::StdError;

/// Aggregate types whose values can be created inside intrinsic or hosted-runtime implementations.
///
/// Compilers retain these layouts when present because their constructors are not represented by a
/// source-level aggregate instruction.
pub const RUNTIME_AGGREGATE_TYPES: &[&str] = &[
    "Std.Json.JsonValue",
    "Std.Toml.TomlValue",
    "Std.Proc.ProcessOutput",
    "Std.Console.KeyEvent",
    "Std.Console.Event",
    "Std.Console.Color",
    "Std.Console.Cell",
    "Std.Graph.Application",
    "Std.Graph.Size",
    "Std.Graph.Event",
];

/// Constructs aggregate values against the layouts of the executing program.
pub trait AggregateFactory {
    /// Construct one record from values in declaration order.
    ///
    /// # Errors
    ///
    /// Returns an internal runtime diagnostic when the type or field layout is unavailable.
    fn record(
        &self,
        type_name: &str,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, StdError>;

    /// Construct one enum variant from associated values in declaration order.
    ///
    /// # Errors
    ///
    /// Returns an internal runtime diagnostic when the type, variant, or field layout is unavailable.
    fn enumeration(
        &self,
        type_name: &str,
        variant: &str,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, StdError>;
}
