# AP51.4: Standalone transport and ACP attachments

Package: [AP51](README.md). Status: Complete.

## Standalone serving

Provide the local Streamable HTTP transport verified in AP51.1 through the
official SDK. An FPAS program creates, registers and serves tools without
initializing ACP, launching an agent or constructing ACP protocol values.
The listener belongs to the hosting application and reaches its live tool state.

Bind to loopback and own the address, access identities, connections, body
limits and shutdown. Use the verified MCP revision/lifecycle through SDK
facilities. Generic remote hosting, managed OAuth and additional transports
are outside this package unless concrete target evidence requires them.

The native implementation in crates/fpas-mcp/src/transport/ provides
authenticated loopback publications with independent endpoint identities,
streamed-body and socket limits, connection deadlines and joined shutdown.
Credentials are fresh per publication and remain outside URLs and diagnostics.
Its close operation preserves caller-owned servers and their other publications.
The public FPAS interface owns publication, confidential access, serving and
joined cleanup. ACP uses fresh publications during normal session setup.

## ACP registration and attachment

`Acp.RegisterTool` creates or uses the connection-owned convenience server and
delegates to the same registration implementation as `Mcp.RegisterTool`.
`Acp.AttachMcpServer` borrows an existing server and reuses its catalog and
handlers without copying or registering them again.

Both paths advertise their descriptors during normal ACP session setup.
Check transport capabilities before sending a descriptor. Define name-conflict
handling when callers also supply MCP descriptors explicitly. Registration,
attachment and setup failure release reservations created solely for that
operation and leave caller-owned servers usable.

Create a fresh attachment before each setup request. Its identity scopes access
and request context while setup is pending, then binds to the returned session.
Failed setup retires it. Closing one session invalidates its attachment and
outstanding work while other attachments remain live. Applications explicitly
choose any shared state; a common catalog does not grant cross-session authority.

| Operation | Lifetime result |
| --- | --- |
| Detach or close an ACP session | Retire its attachment and settle its outstanding work |
| Close the ACP connection | Retire its attachments and close its owned convenience server |
| Detach a borrowed server | Preserve caller ownership and independent serving |
| Close an MCP server explicitly | Stop admission and invalidate every attachment to it |
| Reopen or create another session | Allocate fresh identities; terminal reply rights remain invalid |

Standalone client access has an explicit policy separate from ACP attachment
credentials. Retiring an attachment revokes its access to the catalog.
Keep credentials out of logs and diagnostics.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/transport/` | Official SDK HTTP adapter, bounds and listener lifetime |
| Create | `crates/fpas-vm/src/vm/hosted/mcp/acp.rs` and tests | Existing-server attachment and ACP-owned convenience server |
| Modify | `crates/fpas-acp/src/connection/client/sessions.rs`, ACP setup/lifetime owners | Advertisement, capability checks and attachment retirement |
| Inspect/reuse | `crates/fpas-vm/src/vm/hosted/net/listeners.rs`, `crates/fpas-vm/src/vm/hosted/server/` | Listener and shutdown ownership |
| Inspect as needed | `crates/fpas-acp/src/connection/mcp.rs` | Existing carrier routing if target evidence requires that transport |
| Create/modify | `docs/pascal/std/ai/mcp/transports.md`, `docs/pascal/std/ai/acp/` | Independent serving and ACP registration/attachment |

## Acceptance

Independent MCP peers discover/call tools without ACP. ACP fixtures verify
automatic advertisement, explicit-descriptor conflicts, unsupported transport
capabilities, failed setup, several sessions and detach/close ordering.
Borrowed servers remain usable after ACP closes. ACP-owned servers release
listeners and handlers. Retired credentials and responders remain invalid
after reopen. AP51.6 verifies real Codex interoperability.
