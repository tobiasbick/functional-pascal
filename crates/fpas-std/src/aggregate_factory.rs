//! Runtime-owned construction of typed record and enum values returned by intrinsics.

use fpas_bytecode::{SourceLocation, Value};

use crate::StdError;

/// Aggregate types whose values can be created inside intrinsic or hosted-runtime implementations.
///
/// Compilers retain these layouts when present because their constructors are not represented by a
/// source-level aggregate instruction.
pub const RUNTIME_AGGREGATE_TYPES: &[&str] = &[
    "Std.Acp.Options",
    "Std.Acp.Limits",
    "Std.Acp.ExperimentalOptions",
    "Std.Acp.ProtocolVersion",
    "Std.Acp.MessageConnection",
    "Std.Acp.ConnectionState",
    "Std.Acp.Lifecycle",
    "Std.Acp.Event",
    "Std.Acp.EventPayload",
    "Std.Acp.ClosedStatus",
    "Std.Acp.ProcessExitStatus",
    "Std.Acp.Types.AcpError",
    "Std.Acp.Types.RawJson",
    "Std.Acp.Types.UInt64",
    "Std.Acp.Types.ProtocolError",
    "Std.Errors.HostError",
    "Std.Fs.FsError",
    "Std.Proc.ProcError",
    "Std.Parse.ParseError",
    "Std.Json.JsonError",
    "Std.Toml.TomlError",
    "Std.Crypto.CryptoError",
    "Std.Encoding.EncodingError",
    "Std.Tasks.TaskError",
    "Std.Net.NetError",
    "Std.Server.ServerError",
    "Std.Json.JsonValue",
    "Std.Net.NetworkAddress",
    "Std.Toml.TomlValue",
    "Std.Proc.ProcessOutput",
    "Std.Proc.ProcessOptions",
    "Std.Proc.EnvironmentVariable",
    "Std.Console.KeyEvent",
    "Std.Console.ConsoleEvent",
    "Std.Console.Color",
    "Std.Console.Cell",
];

/// Native aggregate names retained even when only a payload decoder constructs them.
pub fn runtime_aggregate_types() -> &'static [String] {
    static NAMES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
        let mut names = RUNTIME_AGGREGATE_TYPES
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>();
        for version in [fpas_acp::schema::Version::V1, fpas_acp::schema::Version::V2] {
            if let Ok(catalog) = fpas_acp::binding::catalog(version) {
                names.extend(
                    catalog
                        .declarations
                        .values()
                        .filter(|declaration| {
                            !matches!(declaration.shape, fpas_acp::binding::Shape::Scalar(_))
                        })
                        .map(|declaration| declaration.name.clone()),
                );
                let module = if version == fpas_acp::schema::Version::V1 {
                    "Std.Acp.Types"
                } else {
                    "Std.Acp.V2"
                };
                names.extend(
                    fpas_acp::binding::ROOTS
                        .into_iter()
                        .map(|root| format!("{module}.Decoded{root}")),
                );
            }
        }
        names
    });
    &NAMES
}

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
