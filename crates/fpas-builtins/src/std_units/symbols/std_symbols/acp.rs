//! Delivered ACP value operations; connection operations are registered separately.

std_symbol!(STD_ACP_PARSE_RAW_JSON = "Std.Ai.Acp.Types.ParseRawJson");
std_symbol!(STD_ACP_RAW_JSON_TEXT = "Std.Ai.Acp.Types.RawJsonText");
std_symbol!(STD_ACP_RAW_JSON_MEMBER = "Std.Ai.Acp.Types.RawJsonMember");
std_symbol!(STD_ACP_PARSE_UINT64 = "Std.Ai.Acp.Types.ParseUInt64");
std_symbol!(STD_ACP_UINT64_TEXT = "Std.Ai.Acp.Types.UInt64Text");
std_symbol!(STD_ACP_UINT64_FROM_INTEGER = "Std.Ai.Acp.Types.UInt64FromInteger");
std_symbol!(STD_ACP_UINT64_TO_INTEGER = "Std.Ai.Acp.Types.UInt64ToInteger");

pub(in crate::std_units) const STD_ACP_SYMBOLS: &[&str] = &[
    "Std.Ai.Acp.RegisterTool",
    "Std.Ai.Acp.AttachMcpServer",
    "Std.Ai.Acp.DetachMcpServer",
    "Std.Ai.Acp.ServeTools",
    "Std.Ai.Acp.ToolDefinition",
    "Std.Ai.Acp.ToolHandler",
    "Std.Ai.Acp.ToolResult",
    "Std.Ai.Acp.ToolContext",
    "Std.Ai.Acp.ToolJson",
    "Std.Ai.Acp.McpError",
    "Std.Ai.Acp.DefaultOptions",
    "Std.Ai.Acp.RegisterMcpEndpoint",
    "Std.Ai.Acp.RemoveMcpEndpoint",
    "Std.Ai.Acp.ConnectMessages",
    "Std.Ai.Acp.FeedMessage",
    "Std.Ai.Acp.TryTakeMessage",
    "Std.Ai.Acp.CloseInput",
    "Std.Ai.Acp.ConnectProcess",
    "Std.Ai.Acp.Send",
    "Std.Ai.Acp.SendMcp",
    "Std.Ai.Acp.Notify",
    "Std.Ai.Acp.TryReceive",
    "Std.Ai.Acp.Receive",
    "Std.Ai.Acp.Respond",
    "Std.Ai.Acp.RespondError",
    "Std.Ai.Acp.CancelRequest",
    "Std.Ai.Acp.State",
    "Std.Ai.Acp.Close",
    "Std.Ai.Acp.Release",
];

pub(in crate::std_units) const STD_ACP_TYPES_SYMBOLS: &[&str] = &[
    STD_ACP_PARSE_RAW_JSON,
    STD_ACP_RAW_JSON_TEXT,
    STD_ACP_RAW_JSON_MEMBER,
    STD_ACP_PARSE_UINT64,
    STD_ACP_UINT64_TEXT,
    STD_ACP_UINT64_FROM_INTEGER,
    STD_ACP_UINT64_TO_INTEGER,
    "Std.Ai.Acp.Types.DecodeClientRequest",
    "Std.Ai.Acp.Types.EncodeClientRequest",
    "Std.Ai.Acp.Types.DecodeAgentResponse",
    "Std.Ai.Acp.Types.EncodeAgentResponse",
    "Std.Ai.Acp.Types.DecodeClientNotification",
    "Std.Ai.Acp.Types.EncodeClientNotification",
    "Std.Ai.Acp.Types.DecodeAgentRequest",
    "Std.Ai.Acp.Types.EncodeAgentRequest",
    "Std.Ai.Acp.Types.DecodeClientResponse",
    "Std.Ai.Acp.Types.EncodeClientResponse",
    "Std.Ai.Acp.Types.DecodeAgentNotification",
    "Std.Ai.Acp.Types.EncodeAgentNotification",
    "Std.Ai.Acp.Types.DecodeProtocolLevelNotification",
    "Std.Ai.Acp.Types.EncodeProtocolLevelNotification",
];
pub(in crate::std_units) const STD_ACP_HOST_SYMBOLS: &[&str] = &[
    "Std.Ai.Acp.Host.ReadTextFile",
    "Std.Ai.Acp.Host.WriteTextFile",
    "Std.Ai.Acp.Host.CreateTerminal",
    "Std.Ai.Acp.Host.TerminalOutput",
    "Std.Ai.Acp.Host.WaitForTerminalExit",
    "Std.Ai.Acp.Host.KillTerminal",
    "Std.Ai.Acp.Host.ReleaseTerminal",
    "Std.Ai.Acp.Host.RunTerminalAuthentication",
    "Std.Ai.Acp.Host.V2RunTerminalAuthentication",
    "Std.Ai.Acp.Host.ValidateElicitation",
    "Std.Ai.Acp.Host.ElicitationDefaults",
];

pub(in crate::std_units) const STD_ACP_V2_SYMBOLS: &[&str] = &[
    "Std.Ai.Acp.V2.Send",
    "Std.Ai.Acp.V2.SendMcp",
    "Std.Ai.Acp.V2.Notify",
    "Std.Ai.Acp.V2.Respond",
    "Std.Ai.Acp.V2.DecodeClientRequest",
    "Std.Ai.Acp.V2.EncodeClientRequest",
    "Std.Ai.Acp.V2.DecodeAgentResponse",
    "Std.Ai.Acp.V2.EncodeAgentResponse",
    "Std.Ai.Acp.V2.DecodeClientNotification",
    "Std.Ai.Acp.V2.EncodeClientNotification",
    "Std.Ai.Acp.V2.DecodeAgentRequest",
    "Std.Ai.Acp.V2.EncodeAgentRequest",
    "Std.Ai.Acp.V2.DecodeClientResponse",
    "Std.Ai.Acp.V2.EncodeClientResponse",
    "Std.Ai.Acp.V2.DecodeAgentNotification",
    "Std.Ai.Acp.V2.EncodeAgentNotification",
    "Std.Ai.Acp.V2.DecodeProtocolLevelNotification",
    "Std.Ai.Acp.V2.EncodeProtocolLevelNotification",
];
