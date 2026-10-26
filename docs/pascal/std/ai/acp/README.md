# `Std.Acp`

Native ACP connections, typed messages and deferred replies. Import `Std.Acp.Types` for
stable v1 protocol values and payload codecs, `Std.Acp.Experimental` for the
pinned v1 extensions, or `Std.Acp.V2` for the separate v2 protocol family.

`Connection`, `RequestToken`, `Responder`, `McpEndpoint` and `MessageEndpoint` are opaque,
VM-owned identities. Copies alias the same state. Connections use the pinned
Rust SDK on one native executor per VM; no FPAS closure runs on that executor.

See [connections and transports](connections.md) for setup, event ownership,
limits, cooperative waits and cleanup, [sessions](sessions.md) for stable v1
initialization and capability policy, and [content](content.md) for streaming
updates. [Host adapters](host.md) perform explicitly admitted file/terminal work;
[authentication](authentication.md) describes separate interactive login.

## Application tools

`Std.Mcp` registers reusable application tools. ACP can own a convenience server
or attach a caller-owned server. Both use the same schema validation and FPAS
callback execution described in [Std.Mcp](../mcp/README.md).

| Kind | Signature | Description |
| --- | --- | --- |
| function | `RegisterTool(Connection, Definition, Handler)` | Registers an FPAS handler in this connection's convenience MCP server. |
| function | `AttachMcpServer(Connection, Server)` | Borrows a reusable MCP server for future session setup requests. |
| function | `DetachMcpServer(Connection, Server)` | Retires and joins this connection's publications of a borrowed server. |
| function | `ServeTools(Connection, Cancellation)` | Dispatches convenience-server calls on the calling FPAS task until close or cancellation. |

These operations return `result of (boolean, Std.Mcp.Types.McpError)`.
`ToolDefinition`, `ToolHandler`, `ToolResult`, `ToolContext`, `ToolJson` and
`McpError` alias the shared MCP types. Import `Std.Mcp` for JSON/result helpers.
Registration remains mutable until the first serve or session setup. A borrowed
server freezes when attached and is served through `Mcp.Serve` or `Mcp.ServeOne`.

Session setup automatically advertises a freshly authenticated loopback HTTP
endpoint per server. Admission checks the peer's HTTP MCP capability and rejects
conflicting descriptor names. Tools become callable after a successful setup
response binds the endpoint to its session. Every accepted request retains that
connection/session identity. Failed setup retires its endpoints. Admitting a
session-close request retires its publications immediately; a rejected close
cannot reactivate retired MCP rights. Connection close joins all publications and
closes its convenience server. Borrowed servers remain owned by their caller.

See [protocol values](types.md), [experimental features and MCP endpoints](experimental.md),
[v2 negotiation and asynchronous prompts](v2.md), and the
[console examples](../../../../../examples/network/acp-client/README.md).
