//! Shared MCP types and independent server operations.

std_symbol!(STD_MCP_DEFAULT_LIMITS = "Std.Ai.Mcp.DefaultLimits");
std_symbol!(STD_MCP_DEFAULT_HTTP_OPTIONS = "Std.Ai.Mcp.DefaultHttpOptions");
std_symbol!(STD_MCP_CREATE = "Std.Ai.Mcp.Create");
std_symbol!(STD_MCP_REGISTER_TOOL = "Std.Ai.Mcp.RegisterTool");
std_symbol!(STD_MCP_PUBLISH = "Std.Ai.Mcp.Publish");
std_symbol!(STD_MCP_ACCESS = "Std.Ai.Mcp.Access");
std_symbol!(STD_MCP_CLOSE_ENDPOINT = "Std.Ai.Mcp.CloseEndpoint");
std_symbol!(STD_MCP_CLOSE = "Std.Ai.Mcp.Close");
std_symbol!(STD_MCP_RELEASE = "Std.Ai.Mcp.Release");
std_symbol!(STD_MCP_PARSE_JSON = "Std.Ai.Mcp.ParseJson");
std_symbol!(STD_MCP_JSON_MEMBER = "Std.Ai.Mcp.JsonMember");
std_symbol!(STD_MCP_PARSE_TOOL_RESULT = "Std.Ai.Mcp.ParseToolResult");
std_symbol!(STD_MCP_TEXT_RESULT = "Std.Ai.Mcp.TextResult");
std_symbol!(STD_MCP_ERROR_RESULT = "Std.Ai.Mcp.ErrorResult");
std_symbol!(STD_MCP_STRUCTURED_RESULT = "Std.Ai.Mcp.StructuredResult");
std_symbol!(STD_MCP_IS_CANCELLED = "Std.Ai.Mcp.IsCancelled");
std_symbol!(STD_MCP_SERVE_ONE = "Std.Ai.Mcp.ServeOne");
std_symbol!(STD_MCP_SERVE = "Std.Ai.Mcp.Serve");

pub(in crate::std_units) const STD_MCP_SYMBOLS: &[&str] = &[
    "Std.Ai.Mcp.Server",
    "Std.Ai.Mcp.Endpoint",
    "Std.Ai.Mcp.Limits",
    "Std.Ai.Mcp.HttpOptions",
    "Std.Ai.Mcp.RawJson",
    "Std.Ai.Mcp.ToolDefinition",
    "Std.Ai.Mcp.ToolResult",
    "Std.Ai.Mcp.RequestContext",
    "Std.Ai.Mcp.HttpHeader",
    "Std.Ai.Mcp.HttpAccess",
    "Std.Ai.Mcp.ToolHandler",
    "Std.Ai.Mcp.McpError",
    STD_MCP_DEFAULT_LIMITS,
    STD_MCP_DEFAULT_HTTP_OPTIONS,
    STD_MCP_CREATE,
    STD_MCP_REGISTER_TOOL,
    STD_MCP_PUBLISH,
    STD_MCP_ACCESS,
    STD_MCP_CLOSE_ENDPOINT,
    STD_MCP_CLOSE,
    STD_MCP_RELEASE,
    STD_MCP_PARSE_JSON,
    STD_MCP_JSON_MEMBER,
    STD_MCP_PARSE_TOOL_RESULT,
    STD_MCP_TEXT_RESULT,
    STD_MCP_ERROR_RESULT,
    STD_MCP_STRUCTURED_RESULT,
    STD_MCP_IS_CANCELLED,
    STD_MCP_SERVE_ONE,
    STD_MCP_SERVE,
];

pub(in crate::std_units) const STD_MCP_TYPES_SYMBOLS: &[&str] = &[
    "Std.Ai.Mcp.Types.RawJson",
    "Std.Ai.Mcp.Types.ToolDefinition",
    "Std.Ai.Mcp.Types.ToolResult",
    "Std.Ai.Mcp.Types.RequestContext",
    "Std.Ai.Mcp.Types.HttpHeader",
    "Std.Ai.Mcp.Types.HttpAccess",
    "Std.Ai.Mcp.Types.ToolHandler",
    "Std.Ai.Mcp.Types.McpError",
];
