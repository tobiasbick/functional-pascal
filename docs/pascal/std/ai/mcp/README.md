# Std.Ai.Mcp

Registers FPAS functions as MCP tools and publishes them through authenticated
loopback HTTP. Import with `uses Std.Ai.Mcp as Mcp;`. The library owns protocol
handling and invokes registered functions on the task executing Serve or ServeOne.
It requires no ACP connection or agent process.

## Quick reference

| Kind | Signature | Description |
|------|-----------|-------------|
| type | `Server` | Opaque VM-local catalog and handler identity |
| type | `Endpoint` | Independently authenticated HTTP publication |
| type | `Limits` | Tools, CatalogBytes, MessageBytes, PendingCalls, PendingBytes and Endpoints |
| type | `HttpOptions` | Address, Connections, RequestMillis and ShutdownMillis |
| function | `DefaultLimits(): Limits` | Returns bounded catalog and queue defaults |
| function | `DefaultHttpOptions(): HttpOptions` | Returns loopback and transport defaults |
| function | `Create(Limits: Limits): result of (Server, McpError)` | Creates an unpublished registration owner |
| function | `RegisterTool(Server: Server; Definition: ToolDefinition; Handler: ToolHandler): result of (boolean, McpError)` | Validates a definition and retains its FPAS handler |
| function | `Publish(Server: Server; Options: HttpOptions): result of (Endpoint, McpError)` | Freezes registration and opens an authenticated route |
| function | `Access(Endpoint: Endpoint): result of (HttpAccess, McpError)` | Returns confidential URL/header access for an intended consumer |
| function | `CloseEndpoint(Endpoint: Endpoint): result of (boolean, McpError)` | Retires a publication and joins physical cleanup |
| function | `Close(Server: Server): result of (boolean, McpError)` | Cancels calls and joins every publication while preserving terminal identity |
| function | `Release(Server: Server): result of (boolean, McpError)` | Closes the server and releases retained handlers |
| function | `ParseJson(Text: string): result of (RawJson, McpError)` | Checks bounded JSON without rounding arbitrary numbers |
| function | `JsonMember(Value: RawJson; Name: string): result of (option of RawJson, McpError)` | Extracts a detached object member |
| function | `ParseToolResult(Value: RawJson): result of (ToolResult, McpError)` | Checks the SDK tool-result envelope |
| function | `TextResult(Text: string): ToolResult` | Produces successful text content |
| function | `ErrorResult(Text: string): ToolResult` | Produces a tool failure with isError set |
| function | `StructuredResult(Value: RawJson): result of (ToolResult, McpError)` | Produces structured JSON and matching text content |
| function | `IsCancelled(Context: RequestContext): boolean` | Observes cancellation or a terminal call |
| function | `ServeOne(Server: Server; TimeoutMillis: option of integer; Cancellation: option of CancellationToken): result of (boolean, McpError)` | Receives and invokes one handler, retaining exclusive serving ownership |
| function | `Serve(Server: Server; Cancellation: option of CancellationToken): result of (boolean, McpError)` | Serves until closure or cancellation without an application protocol loop |

The common types are defined in [Std.Ai.Mcp.Types](types.md) and re-exported here.
CancellationToken belongs to Std.Tasks. HttpAccess contains credentials: supply
it only to the intended consumer and keep it out of logs.

## Ownership and execution

Register tools before the first Publish or Serve operation. The catalog is
fixed thereafter. Duplicate names and invalid definitions leave registration
unchanged. Definitions use explicit schemas; FPAS signature reflection is not
performed. Input schemas require an object root and cannot retrieve external
references.

One serving task owns the native reader, including while a handler suspends.
A competing serving operation returns Busy. Handler frames use the ordinary
VM callback path and remain visible to the debugger. Captured mutable values
remain with their owning task. Registration never transfers mutable authority.
Forward application requests over bounded channels when another task owns state.

A handler has the type function(Context: RequestContext; Arguments: RawJson):
result of (ToolResult, McpError). Apply application policy before effects.
Ok(ErrorResult(...)) is a tool failure. Error(McpError) becomes a protocol
failure. A VM failure unwinds the handler, settles its outstanding reply and
releases reader ownership. No effect is retried automatically.

ServeOne returns false when its receive timeout elapses without work. Serve
finishes normally when the server closes. A cancellation token produces
Cancelled. Invalid handler output settles the remote call and reports
InvalidResult locally. Cancellation cannot undo an application effect.

Closing an endpoint preserves the server and other endpoints. Closing the
server revokes every endpoint and reply right. Release also drops retained
handler roots; callables and captures still held by application code remain
ordinary application values. Handles and call identities never revive.

## Bounds

Default limits are 128 tools, 1 MiB of catalog definitions, 1 MiB per message,
64 pending calls, 8 MiB of retained call/context data and 16 endpoints.
HTTP defaults to 127.0.0.1:0, 64 sockets, a 30-second request deadline and a
five-second shutdown deadline. Host must match the actual bound address and
port. Every present Origin header is rejected.

The VM permits 16 servers. Its native HTTP executor bounds publication openings
and active publications independently. Close remains available when ordinary
admission is full and reports transport cleanup failures.
