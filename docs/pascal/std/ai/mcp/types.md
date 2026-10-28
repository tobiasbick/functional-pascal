# Std.Ai.Mcp.Types

Shared tool values, handler context and operational errors. These declarations
are independent of ACP and are re-exported by Std.Ai.Mcp.

## Values

| Kind | Signature | Description |
|------|-----------|-------------|
| type | `RawJson` | Text carrying bounded JSON and exact numeric representations |
| type | `ToolDefinition` | Name, Description, InputSchema, optional OutputSchema, Annotations and Meta |
| type | `ToolResult` | SDK tool-result envelope in Payload |
| type | `RequestContext` | Server, AttachmentId, CallId, RequestId, Meta, ProtocolVersion, ConnectionId and SessionId |
| type | `ToolHandler` | Function taking RequestContext and RawJson and returning result of ToolResult and McpError |
| type | `HttpHeader` | Name and confidential Value |
| type | `HttpAccess` | Url and Headers for one intended consumer |
| type | `McpError` | Nominal registration, serving and transport failure |

RawJson.Text is available for explicit JSON processing. Native operations
validate values even when a caller constructs a record directly. ToolResult
preserves SDK content, structuredContent, isError and metadata; ParseToolResult
admits rich SDK results without a separate FPAS content protocol.

CallId is an internal call identity, distinct from the remote JSON-RPC RequestId.
AttachmentId is scoped by Server. ConnectionId and SessionId are absent for
standalone publications. Caller metadata supplies no application authority.
IsCancelled is true after any terminal reply or cancellation.

## Errors

McpError declares InvalidArgument(Argument), InvalidSchema(Schema), DuplicateTool,
UnknownTool, InvalidArguments, InvalidResult, LimitExceeded(Resource, Limit),
Busy, Closed, Transport(Kind), Cancelled, Timeout and PermissionDenied.
Ordinary Message output omits associated data. Match variants and inspect their
fields explicitly when diagnostics require them.
