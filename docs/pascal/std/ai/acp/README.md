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

See [protocol values](types.md), [experimental features and MCP endpoints](experimental.md),
[v2 negotiation and asynchronous prompts](v2.md), and the
[console examples](../../../../../examples/network/acp-client/README.md).
