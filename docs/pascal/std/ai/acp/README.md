# `Std.Acp`

ACP namespace and opaque connection identities. Import `Std.Acp.Types` for
stable v1 protocol values and payload codecs, `Std.Acp.Experimental` for the
pinned v1 extensions, or `Std.Acp.V2` for the separate v2 protocol family.

`Connection`, `RequestToken`, and `Responder` are opaque identity types reserved
for the connection API. AP50.1 supplies detached values and codecs; connection
creation, event delivery and host effects are implemented by subsequent AP50
packages.

See [protocol values](types.md), [host payload aliases](host.md),
[experimental values](experimental.md), and [v2 values](v2.md).
