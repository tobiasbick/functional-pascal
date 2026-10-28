//! Native aggregate declarations retained by compilation and image admission.

/// Aggregate types whose values can be created inside intrinsic or hosted-runtime implementations.
///
/// Compilers retain these layouts when present because their constructors are not represented by a
/// source-level aggregate instruction.
pub const RUNTIME_AGGREGATE_TYPES: &[&str] = &[
    "Std.Mcp.Limits",
    "Std.Mcp.HttpOptions",
    "Std.Mcp.Types.RawJson",
    "Std.Mcp.Types.ToolDefinition",
    "Std.Mcp.Types.ToolResult",
    "Std.Mcp.Types.RequestContext",
    "Std.Mcp.Types.HttpHeader",
    "Std.Mcp.Types.HttpAccess",
    "Std.Mcp.Types.McpError",
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
        for version in [
            fpas_acp_model::schema::Version::V1,
            fpas_acp_model::schema::Version::V2,
        ] {
            if let Ok(catalog) = fpas_acp_model::binding::catalog(version) {
                names.extend(
                    catalog
                        .declarations
                        .values()
                        .filter(|declaration| {
                            !matches!(declaration.shape, fpas_acp_model::binding::Shape::Scalar(_))
                        })
                        .map(|declaration| declaration.name.clone()),
                );
                let module = if version == fpas_acp_model::schema::Version::V1 {
                    "Std.Acp.Types"
                } else {
                    "Std.Acp.V2"
                };
                names.extend(
                    fpas_acp_model::binding::ROOTS
                        .into_iter()
                        .map(|root| format!("{module}.Decoded{root}")),
                );
            }
        }
        names
    });
    &NAMES
}
