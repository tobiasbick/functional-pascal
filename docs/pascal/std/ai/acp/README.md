# `Std.Acp`

Native ACP connections, typed messages and deferred replies. Import `Std.Acp.Types` for
stable v1 protocol values and payload codecs, `Std.Acp.Experimental` for the
pinned v1 extensions, or `Std.Acp.V2` for the separate v2 protocol family.

`Connection`, `RequestToken`, `Responder` and `MessageEndpoint` are opaque,
VM-owned identities. Copies alias the same state. Connections use the pinned
Rust SDK on one native executor per VM; no FPAS closure runs on that executor.

See [connections and transports](connections.md) for setup, event ownership,
limits, cooperative waits and cleanup. Protocol initialization/session policy
and opt-in host adapters remain separate AP50 work packages.

See [protocol values](types.md), [host payload aliases](host.md),
[experimental values](experimental.md), and [v2 values](v2.md).
