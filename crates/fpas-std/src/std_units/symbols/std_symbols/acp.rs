//! Delivered ACP value operations; connection operations are registered separately.

std_symbol!(STD_ACP_PARSE_RAW_JSON = "Std.Acp.Types.ParseRawJson");
std_symbol!(STD_ACP_RAW_JSON_TEXT = "Std.Acp.Types.RawJsonText");
std_symbol!(STD_ACP_RAW_JSON_MEMBER = "Std.Acp.Types.RawJsonMember");
std_symbol!(STD_ACP_PARSE_UINT64 = "Std.Acp.Types.ParseUInt64");
std_symbol!(STD_ACP_UINT64_TEXT = "Std.Acp.Types.UInt64Text");
std_symbol!(STD_ACP_UINT64_FROM_INTEGER = "Std.Acp.Types.UInt64FromInteger");
std_symbol!(STD_ACP_UINT64_TO_INTEGER = "Std.Acp.Types.UInt64ToInteger");

pub(in crate::std_units) const STD_ACP_SYMBOLS: &[&str] = &[
    "Std.Acp.DefaultOptions",
    "Std.Acp.RegisterMcpEndpoint",
    "Std.Acp.RemoveMcpEndpoint",
    "Std.Acp.ConnectMessages",
    "Std.Acp.FeedMessage",
    "Std.Acp.TryTakeMessage",
    "Std.Acp.CloseInput",
    "Std.Acp.ConnectProcess",
    "Std.Acp.Send",
    "Std.Acp.SendMcp",
    "Std.Acp.Notify",
    "Std.Acp.TryReceive",
    "Std.Acp.Receive",
    "Std.Acp.Respond",
    "Std.Acp.RespondError",
    "Std.Acp.CancelRequest",
    "Std.Acp.State",
    "Std.Acp.Close",
    "Std.Acp.Release",
];

pub(in crate::std_units) const STD_ACP_TYPES_SYMBOLS: &[&str] = &[
    STD_ACP_PARSE_RAW_JSON,
    STD_ACP_RAW_JSON_TEXT,
    STD_ACP_RAW_JSON_MEMBER,
    STD_ACP_PARSE_UINT64,
    STD_ACP_UINT64_TEXT,
    STD_ACP_UINT64_FROM_INTEGER,
    STD_ACP_UINT64_TO_INTEGER,
    "Std.Acp.Types.DecodeClientRequest",
    "Std.Acp.Types.EncodeClientRequest",
    "Std.Acp.Types.DecodeAgentResponse",
    "Std.Acp.Types.EncodeAgentResponse",
    "Std.Acp.Types.DecodeClientNotification",
    "Std.Acp.Types.EncodeClientNotification",
    "Std.Acp.Types.DecodeAgentRequest",
    "Std.Acp.Types.EncodeAgentRequest",
    "Std.Acp.Types.DecodeClientResponse",
    "Std.Acp.Types.EncodeClientResponse",
    "Std.Acp.Types.DecodeAgentNotification",
    "Std.Acp.Types.EncodeAgentNotification",
    "Std.Acp.Types.DecodeProtocolLevelNotification",
    "Std.Acp.Types.EncodeProtocolLevelNotification",
];
pub(in crate::std_units) const STD_ACP_HOST_SYMBOLS: &[&str] = &[
    "Std.Acp.Host.ReadTextFile",
    "Std.Acp.Host.WriteTextFile",
    "Std.Acp.Host.CreateTerminal",
    "Std.Acp.Host.TerminalOutput",
    "Std.Acp.Host.WaitForTerminalExit",
    "Std.Acp.Host.KillTerminal",
    "Std.Acp.Host.ReleaseTerminal",
    "Std.Acp.Host.RunTerminalAuthentication",
    "Std.Acp.Host.V2RunTerminalAuthentication",
    "Std.Acp.Host.ValidateElicitation",
    "Std.Acp.Host.ElicitationDefaults",
];

pub(in crate::std_units) const STD_ACP_V2_SYMBOLS: &[&str] = &[
    "Std.Acp.V2.Send",
    "Std.Acp.V2.SendMcp",
    "Std.Acp.V2.Notify",
    "Std.Acp.V2.Respond",
    "Std.Acp.V2.DecodeClientRequest",
    "Std.Acp.V2.EncodeClientRequest",
    "Std.Acp.V2.DecodeAgentResponse",
    "Std.Acp.V2.EncodeAgentResponse",
    "Std.Acp.V2.DecodeClientNotification",
    "Std.Acp.V2.EncodeClientNotification",
    "Std.Acp.V2.DecodeAgentRequest",
    "Std.Acp.V2.EncodeAgentRequest",
    "Std.Acp.V2.DecodeClientResponse",
    "Std.Acp.V2.EncodeClientResponse",
    "Std.Acp.V2.DecodeAgentNotification",
    "Std.Acp.V2.EncodeAgentNotification",
    "Std.Acp.V2.DecodeProtocolLevelNotification",
    "Std.Acp.V2.EncodeProtocolLevelNotification",
];
