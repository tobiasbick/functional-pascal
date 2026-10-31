//! Native aggregate declarations retained by compilation and image admission.

/// Aggregate types whose values can be created inside intrinsic or hosted-runtime implementations.
///
/// Compilers retain these layouts when present because their constructors are not represented by a
/// source-level aggregate instruction.
pub const RUNTIME_AGGREGATE_TYPES: &[&str] = &[
    "Std.Ai.Tokenizer.TokenizerError",
    "Std.Ai.Mcp.Limits",
    "Std.Ai.Mcp.HttpOptions",
    "Std.Ai.Mcp.Types.RawJson",
    "Std.Ai.Mcp.Types.ToolDefinition",
    "Std.Ai.Mcp.Types.ToolResult",
    "Std.Ai.Mcp.Types.RequestContext",
    "Std.Ai.Mcp.Types.HttpHeader",
    "Std.Ai.Mcp.Types.HttpAccess",
    "Std.Ai.Mcp.Types.McpError",
    "Std.Ai.Acp.Options",
    "Std.Ai.Acp.Limits",
    "Std.Ai.Acp.ExperimentalOptions",
    "Std.Ai.Acp.ProtocolVersion",
    "Std.Ai.Acp.MessageConnection",
    "Std.Ai.Acp.ConnectionState",
    "Std.Ai.Acp.Lifecycle",
    "Std.Ai.Acp.Event",
    "Std.Ai.Acp.EventPayload",
    "Std.Ai.Acp.ClosedStatus",
    "Std.Ai.Acp.ProcessExitStatus",
    "Std.Ai.Acp.Types.AcpError",
    "Std.Ai.Acp.Types.RawJson",
    "Std.Ai.Acp.Types.UInt64",
    "Std.Ai.Acp.Types.ProtocolError",
    "Std.Os.Errors.HostError",
    "Std.Io.Fs.FsError",
    "Std.Os.Proc.ProcError",
    "Std.Text.Parse.ParseError",
    "Std.Encoding.Json.JsonError",
    "Std.Encoding.Toml.TomlError",
    "Std.Crypto.CryptoError",
    "Std.Encoding.EncodingError",
    "Std.Tasks.TaskError",
    "Std.Net.Sockets.NetError",
    "Std.Net.Server.ServerError",
    "Std.Encoding.Json.JsonValue",
    "Std.Net.Sockets.NetworkAddress",
    "Std.Encoding.Toml.TomlValue",
    "Std.Os.Proc.ProcessOutput",
    "Std.Os.Proc.ProcessOptions",
    "Std.Os.Proc.EnvironmentVariable",
    "Std.Console.KeyEvent",
    "Std.Console.ConsoleEvent",
    "Std.Console.Color",
    "Std.Console.Cell",
];

/// Whether a case-insensitive aggregate name needs retention for native construction.
/// ACP schemas are initialized only when an object actually contains an ACP aggregate.
pub fn is_runtime_aggregate_type(name: &str) -> bool {
    if RUNTIME_AGGREGATE_TYPES
        .iter()
        .any(|known| known.eq_ignore_ascii_case(name))
    {
        return true;
    }
    const ACP_PREFIX: &str = "Std.Ai.Acp.";
    if !name
        .get(..ACP_PREFIX.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(ACP_PREFIX))
    {
        return false;
    }
    static NAMES: std::sync::LazyLock<std::collections::HashSet<String>> =
        std::sync::LazyLock::new(|| {
            let mut names = std::collections::HashSet::new();
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
                                !matches!(
                                    declaration.shape,
                                    fpas_acp_model::binding::Shape::Scalar(_)
                                )
                            })
                            .map(|declaration| declaration.name.to_ascii_lowercase()),
                    );
                    let module = if version == fpas_acp_model::schema::Version::V1 {
                        "Std.Ai.Acp.Types"
                    } else {
                        "Std.Ai.Acp.V2"
                    };
                    names.extend(
                        fpas_acp_model::binding::ROOTS
                            .into_iter()
                            .map(|root| format!("{module}.Decoded{root}").to_ascii_lowercase()),
                    );
                }
            }
            names
        });
    NAMES.contains(&name.to_ascii_lowercase())
}
