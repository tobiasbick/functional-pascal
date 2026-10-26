# AP51.1: Public contract and target compatibility

Package: [AP51](README.md). Status: Complete.

## Deliverable

Specify independent `Std.Mcp` serving, ACP convenience registration and
attachment of a caller-owned server. Verify the planned local Streamable HTTP
route against the actual Codex ACP adapter. The package README owns decisions
Q58 through Q64.

## Public contract

Define `Std.Mcp` and `Std.Mcp.Types` independently of ACP. Specify server
creation, `Mcp.RegisterTool`, cooperative serving, close and release.
Registration pairs a definition with an FPAS function. Define a uniform handler
type accepting request context and structured arguments and returning a typed
tool result or failure.

Specify `Acp.RegisterTool` as delegation to the same registration implementation
using a connection-owned convenience server. Specify `Acp.AttachMcpServer` for
a caller-owned server. Attachment reuses its existing catalog and handlers.
Both ACP paths advertise a descriptor during the relevant session setup.

Settle these details before AP51.2:

- Complete types, error results and exact operation names beyond the agreed
  `RegisterTool` entry points.
- The serving operation that invokes handlers on a designated FPAS task without
  a caller-written protocol loop, including cancellation and reentry.
- Registration before publication, duplicate names and catalog freezing.
- Context identity and access for pending setup and established sessions,
  including several sessions using one catalog.
- Ownership and reply-right outcomes on detach, failed setup, server closure,
  ACP closure and release.
- Concrete limits for bodies, catalog size, pending requests, retained bytes,
  active handlers and shutdown.

Callbacks follow FPAS task ownership and GC rules. Applications can queue
requests to a task that owns mutable state. Registration cannot grant unsafe
access to another task's mutable captures.

## SDK configuration

Use the official [rmcp crate](https://crates.io/crates/rmcp/3.7.0).
Pin `rmcp` to 3.7.0 with `default-features = false`, `server`, `schemars` and
`transport-streamable-http-server`. Reuse existing crypto/TLS policy where
applicable. The product dependency graph enables no SDK client, HTTP-client or
automatic MRTR feature.

The SDK owns models, serialization, protocol lifecycle and server handling.
Its typed Rust parameter extractors do not validate arbitrary registered JSON
schemas. Use `jsonschema` 0.60.1 with default features disabled,
`arbitrary-precision` enabled and offline compilation. External schema retrieval
is disabled. Both public registration paths share these compiled validators.
FPAS owns conversion, callable registration, scheduling, policy and resource
bounds. Research probes stay under `.temp-data/ap51-research/`.

## Native implementation

`crates/fpas-mcp/` provides the catalog and SDK `ServerHandler` adapter. Native
`Catalog::register` pairs a definition with an application handler identity.
`Server::new` consumes and freezes that catalog. Independent `Endpoint` values
scope request rights. One `Reader` receives detached calls; aliased `Responder`
values consume one reply right. Native HTTP publications own loopback listeners,
accepted sockets, attachment credentials and bounded shutdown. FPAS types,
callback retention and serving operations remain part of the public contract
work.

The native defaults are 128 tools, 1 MiB of serialized catalog definitions,
1 MiB per call/result, 64 pending calls, 8 MiB of retained call representations
and 16 endpoints. Pending limits include claimed calls and request metadata.
Cancellation, close and abandoned responders release admission independently
of an application callback finishing. Applications own effect deduplication.

## Verified local target contract

| Part | Verified configuration |
| --- | --- |
| Codex executable used by the adapter | 0.160.1 |
| ACP adapter | `@agentclientprotocol/codex-acp` 2.2.2 |
| Model | `gpt-6-luna`, low reasoning |
| ACP | Version 1; HTTP supported, MCP-over-ACP and SSE unsupported |
| MCP route | Native authenticated HTTP publication in the hosting application |
| Access | Fresh bearer credential supplied through the ACP HTTP descriptor |
| MCP revision | `2025-06-18` |
| Lifecycle | `initialize`, `notifications/initialized`, `tools/list`, `tools/call` |
| SDK HTTP configuration | `legacy_session_mode = false`, `json_response = true` |

The route supports calls to the native catalog and detached application queue.
Keep SDK negotiation for other peers; this table establishes one tested target
combination. VM-owned authenticated listeners, automatic ACP advertisement and
task-owned FPAS callbacks use this route. Both ACP convenience registration and
borrowed-server registration pass live read/update verification with Luna.

## Local target verification

1. Record Codex and ACP-adapter versions separately. Initialize the real adapter
   and inspect its MCP capabilities.
2. Serve a minimal SDK tool through loopback HTTP and provide its descriptor
   during ACP setup. Prove a real discovery/call reaches the hosting process.
3. Record the negotiated MCP revision and lifecycle, including initialization
   or discovery as implemented by the target and SDK.
4. Define access identity and its relation to standalone callers and each ACP
   setup/session. Verify that the handler can use hosting-application state.
5. Verify standalone serving without ACP and attachment of the same server to
   ACP without transferring ownership or registering its tools again.

Use Luna for live model requests. Live verification runs locally. Account-free
fixtures establish protocol behavior but do not prove Codex interoperability.
If the planned route fails, establish its cause before changing transport or
adding a state-transfer protocol. MCP-over-ACP requires explicit target evidence.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Specify | Package README and AP51.2 through AP51.6 | Exact public contract, ownership, limits and acceptance |
| Inspect | `crates/fpas-acp/src/connection/`, `crates/fpas-vm/src/vm/hosted/acp/` | Session setup, capabilities and native lifetime |
| Inspect | `crates/fpas-vm/src/vm/hosted/server/`, `docs/pascal/agents/concurrency.md` | Callback ownership, serving and shutdown |
| Reuse | `crates/fpas-vm/src/vm/hosted/callbacks/` | Task-owned resumable callback frames and GC roots |
| Inspect | Published `rmcp` sources and Codex ACP adapter | Exact SDK configuration and target behavior |

## Acceptance

The contract covers all three usage paths, borrowed-server survival,
per-attachment isolation and task-owned state. The reduced SDK configuration
builds. Real Codex traffic reaches the selected route and its protocol/lifecycle
is recorded. Keep unverified target details open. Align dependent packages,
check links and run `git diff --check`.
