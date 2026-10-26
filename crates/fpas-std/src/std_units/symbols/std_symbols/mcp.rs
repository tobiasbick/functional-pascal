//! Shared MCP types and independent server operations.

std_symbol!(STD_MCP_DEFAULT_LIMITS = "Std.Mcp.DefaultLimits");
std_symbol!(STD_MCP_DEFAULT_HTTP_OPTIONS = "Std.Mcp.DefaultHttpOptions");
std_symbol!(STD_MCP_CREATE = "Std.Mcp.Create");
std_symbol!(STD_MCP_REGISTER_TOOL = "Std.Mcp.RegisterTool");
std_symbol!(STD_MCP_PUBLISH = "Std.Mcp.Publish");
std_symbol!(STD_MCP_ACCESS = "Std.Mcp.Access");
std_symbol!(STD_MCP_CLOSE_ENDPOINT = "Std.Mcp.CloseEndpoint");
std_symbol!(STD_MCP_CLOSE = "Std.Mcp.Close");
std_symbol!(STD_MCP_RELEASE = "Std.Mcp.Release");
std_symbol!(STD_MCP_PARSE_JSON = "Std.Mcp.ParseJson");
std_symbol!(STD_MCP_JSON_MEMBER = "Std.Mcp.JsonMember");
std_symbol!(STD_MCP_PARSE_TOOL_RESULT = "Std.Mcp.ParseToolResult");
std_symbol!(STD_MCP_TEXT_RESULT = "Std.Mcp.TextResult");
std_symbol!(STD_MCP_ERROR_RESULT = "Std.Mcp.ErrorResult");
std_symbol!(STD_MCP_STRUCTURED_RESULT = "Std.Mcp.StructuredResult");
std_symbol!(STD_MCP_IS_CANCELLED = "Std.Mcp.IsCancelled");
std_symbol!(STD_MCP_SERVE_ONE = "Std.Mcp.ServeOne");
std_symbol!(STD_MCP_SERVE = "Std.Mcp.Serve");

pub(in crate::std_units) const STD_MCP_SYMBOLS: &[&str] = &[
    "Std.Mcp.Server",
    "Std.Mcp.Endpoint",
    "Std.Mcp.Limits",
    "Std.Mcp.HttpOptions",
    "Std.Mcp.RawJson",
    "Std.Mcp.ToolDefinition",
    "Std.Mcp.ToolResult",
    "Std.Mcp.RequestContext",
    "Std.Mcp.HttpHeader",
    "Std.Mcp.HttpAccess",
    "Std.Mcp.ToolHandler",
    "Std.Mcp.McpError",
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
    "Std.Mcp.Types.RawJson",
    "Std.Mcp.Types.ToolDefinition",
    "Std.Mcp.Types.ToolResult",
    "Std.Mcp.Types.RequestContext",
    "Std.Mcp.Types.HttpHeader",
    "Std.Mcp.Types.HttpAccess",
    "Std.Mcp.Types.ToolHandler",
    "Std.Mcp.Types.McpError",
];
