//! Runtime-owned construction of typed record and enum values returned by intrinsics.

use fpas_bytecode::{SourceLocation, Value};

use crate::StdError;

/// Aggregate types whose values can be created inside intrinsic or hosted-runtime implementations.
///
/// Compilers retain these layouts when present because their constructors are not represented by a
/// source-level aggregate instruction.
pub const RUNTIME_AGGREGATE_TYPES: &[&str] = &[
    "Std.Errors.HostError",
    "Std.Fs.FsError",
    "Std.Proc.ProcError",
    "Std.Parse.ParseError",
    "Std.Json.JsonError",
    "Std.Toml.TomlError",
    "Std.Crypto.CryptoError",
    "Std.Tasks.TaskError",
    "Std.Net.NetError",
    "Std.Server.ServerError",
    "Std.Json.JsonValue",
    "Std.Net.NetworkAddress",
    "Std.Toml.TomlValue",
    "Std.Proc.ProcessOutput",
    "Std.Console.KeyEvent",
    "Std.Console.ConsoleEvent",
    "Std.Console.Color",
    "Std.Console.Cell",
];

/// Constructs aggregate values against the layouts of the executing program.
pub trait AggregateFactory {
    /// Construct a native operational error by its shared catalog slot.
    ///
    /// # Errors
    /// Returns an internal diagnostic for missing or invalid layout metadata.
    fn native_error(
        &self,
        family: crate::native_errors::NativeErrorFamily,
        variant: usize,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, StdError> {
        let variant = family.variants().get(variant).ok_or_else(|| {
            crate::error::std_internal_error(
                "Native error catalog slot is unavailable",
                "Rebuild the runtime and compiler together.",
                location,
            )
        })?;
        self.enumeration(family.name(), variant.name, values, location)
    }

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
