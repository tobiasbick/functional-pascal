# AP51: MCP wrapper

Status: Decision pending. Effort: Large; both client and server are required,
with transport and authentication details still open. Completion is tracked in the
[central plan](../README.md#package-status).

## Goal and authorization

Expose the Model Context Protocol to FPAS programs through a native Rust-SDK
wrapper, following the typed values, explicit effects, cooperative waits and
bounded resource ownership used by `Std.Acp`.

The user requested a new AP51 branch and a wrapper analogous to ACP, referencing
the MCP `2026-07-28` introduction. This authorizes the branch and planning.
The user explicitly selected both client and server support (Q58).
The user also requires the official Rust SDK `rmcp`, with an FPAS wrapper around
it analogous to ACP (Q64).
The detailed API and implementation are proposals until agreed. No FPAS syntax
change is currently needed; any concrete syntax change requires user approval.

## Current behavior

- [Std.Acp](../../../pascal/std/ai/acp/README.md) provides ACP clients and
  [experimental MCP carrier endpoints](../../../pascal/std/ai/acp/experimental.md#mcp-carrier-endpoints).
  Those endpoints route opaque MCP messages inside ACP; they are not a standalone
  MCP client or server and do not execute tools automatically.
- There is no `Std.Mcp` unit or `fpas-mcp` crate in the current checkout.
- Existing ACP protocol types and session rules belong to ACP. The process
  registry, task cancellation, VM ownership and API export infrastructure are
  reusable foundations; copying the ACP protocol state machine is not appropriate.
- [Std.Server](../../../pascal/std/network/server.md) already owns cooperative
  listener/task shutdown. MCP serving must integrate with that lifetime contract
  or define an explicit adapter; it must not imply process-exit authority.
- AP50 is delivered on `main`. AP51 has no unfinished AP prerequisite.

## Protocol and SDK evidence

The requested reference is the
[MCP introduction](https://modelcontextprotocol.io/docs/2026-07-28/getting-started/intro).
Its [official source](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/c254060614b5a7d5f40ecbab6234e47c8d74cd5a/docs/docs/2026-07-28/getting-started/intro.mdx)
and the versioned specification establish the planning baseline:

- [`2026-07-28` core](https://modelcontextprotocol.io/specification/2026-07-28/basic):
  requests are self-contained; there is no `initialize`/`initialized` handshake
  or protocol-level session. `server/discover` is optional.
- [Streamable HTTP](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http):
  POST requests receive JSON or request-scoped SSE. There is no legacy GET
  event channel, `Mcp-Session-Id`, or resumable `Last-Event-ID` stream.
- [MRTR](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/mrtr):
  input requests arrive in `input_required` results. Continuation resends the
  original operation with a fresh wire ID, exact opaque `requestState`, and
  matching `inputResponses`.
- [Subscriptions](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/subscriptions)
  use `subscriptions/listen`. Tasks are a separately negotiated extension.
- [Deprecations](https://modelcontextprotocol.io/specification/2026-07-28/deprecated)
  include roots, sampling, logging, dynamic client registration and legacy
  HTTP+SSE. These should not become accidental requirements of a new wrapper.

The [official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk/tree/8aec3891dd07fc537b21c2eb2596235640de9025)
declares `rmcp` 3.7.0, Rust 1.88, and support for this protocol revision in the
inspected source. The official SDK is selected; 3.7.0 is a candidate version,
not a verified dependency pin. AP51.1 must check a released artifact and actual
API behavior. The SDK's default
`serve()` still selects legacy initialization; the wrapper must explicitly
select the modern lifecycle. SDK defaults must not choose FPAS's public contract.

## Agreed implementation boundary

Use the official `modelcontextprotocol/rust-sdk` project and its `rmcp` crate for
both roles. The SDK owns MCP protocol types, serialization, request processing
and supported transport/authentication mechanisms. Call and configure those
facilities rather than implementing a parallel MCP or JSON-RPC engine.

The FPAS layer supplies type/value conversion, public units, native handles,
cooperative waits, application dispatch, resource limits and lifetime adapters
to FPAS processes, streams and listeners. Transport and authorization work
packages describe adapters and application policy around SDK facilities, not
replacement implementations. Reuse SDK validation at its owning boundary;
additional checks address FPAS values, ownership or verified SDK gaps.

If the selected release lacks required functionality, identify the gap and
resolve it through the official SDK or an agreed scope adjustment. A custom
protocol implementation is outside AP51's agreed architecture.

## Proposed contract

The agreed scope includes MCP clients and MCP servers written in FPAS. Proposed
units are `Std.Mcp`, `Std.Mcp.Types` and `Std.Mcp.Server`, implemented by
`crates/fpas-mcp/` and a VM bridge under `hosted/mcp/`. Unit names and handler API
details remain part of Q59; the server role itself is decided.

- Cover the selected modern revision directly, without automatic legacy fallback.
- Offer owned child-process stdio, Streamable HTTP, and a bounded message
  transport for embedding and protocol tests on the client side. Offer explicit
  process stdin/stdout ownership and HTTP listeners on the server side.
- Use typed SDK-backed request/result values while retaining dynamic JSON,
  metadata, unknown extension data and peer error details where the spec permits.
- Expose tools, resources, prompts, completion, progress, subscriptions and MRTR
  elicitation with explicit caller participation. Capability advertisement
  reflects only configured, implemented behavior.
- A local connection handle owns transport/runtime resources, not an MCP
  session. Request tokens, continuation rights and subscriptions have distinct
  identities, cancellation and release rules.
- Preserve caller-controlled pagination and cache metadata. Do not invent a
  shared cache, automatic tool execution, LLM provider, browser launch or retry
  of an uncertain side-effecting request.
- Reuse FPAS process, cancellation and GC ownership foundations. Do not run
  FPAS closures on the SDK executor or block VM workers on native I/O.
- Let FPAS servers provide tools, resources and prompts through explicitly
  dispatched, bounded requests with single-use response rights. Authorize each
  request before application effects, validate modern HTTP headers, and keep
  simultaneous clients and MRTR state isolated without protocol sessions.

## Decisions

The wrapper direction, referenced documentation, both protocol roles and use of
the official Rust SDK are user-specified. Other recommendations in this table
are not user decisions.

| ID | Question | Recommendation | User decision | Affected work packages |
| --- | --- | --- | --- | --- |
| Q58 | Client only, or client and server? | Both roles, with explicit server ownership and dispatch | Client and server | AP51.1–AP51.9 |
| Q59 | Protocol revision, legacy behavior and public API shape? | Modern `2026-07-28`, explicit discovery, no legacy fallback; typed `Std.Mcp`/`Std.Mcp.Types`/`Std.Mcp.Server` and explicit request dispatch | Pending | AP51.1–AP51.9 |
| Q60 | Transport scope? | Client child-process stdio and HTTP; server stdin/stdout and HTTP listeners; bounded message embedding for both; no legacy HTTP+SSE | Pending | AP51.1, AP51.3, AP51.4, AP51.7–AP51.9 |
| Q61 | Core features, deprecated features and extensions? | Core methods/patterns and elicitation for both roles; tasks as explicit opt-in; exclude deprecated roots/sampling/logging, MCP Apps and automatic caching | Pending | AP51.1, AP51.2, AP51.5–AP51.9 |
| Q62 | HTTP authentication scope? | Caller-supplied client credentials; explicit server token validation/authorization configuration and typed auth failures; no implicit OAuth login, token store or authorization server | Pending | AP51.1, AP51.4, AP51.8, AP51.9 |
| Q63 | Should AP51 integrate MCP-over-ACP? | Keep existing carriers unchanged initially; add an adapter only if expressly selected and role/direction compatibility is proven | Pending | AP51.1, AP51.6, AP51.9 |
| Q64 | SDK wrapper or independent MCP implementation? | Official Rust SDK with an FPAS integration layer | Use official `rmcp` for client and server; wrap it as with ACP | AP51.1–AP51.9 |

## Open decisions

Q58 and Q64 are resolved. Q59–Q63 still block the affected implementation: the concrete
handler API, transport policy, deprecated features, OAuth/credential validation
and ACP integration remain open. AP51.7 and AP51.8 own server delivery, including
stdio ownership, HTTP listeners, handler execution, request-state integrity and
server authorization. Both roles are mandatory for package completion.

## Dependencies and work packages

- [ ] [AP51.1: Agree scope and validate the SDK](01-contract-and-sdk.md)
- [ ] [AP51.2: Protocol values and FPAS API registration](02-types-and-codecs.md)
- [ ] [AP51.3: Owned runtime, message transport and stdio](03-runtime-and-stdio.md)
- [ ] [AP51.4: Streamable HTTP and authentication boundary](04-http-and-auth.md)
- [ ] [AP51.5: Core operations and interaction patterns](05-client-features.md)
- [ ] [AP51.6: Extensions and ACP boundary](06-extensions-and-acp.md)
- [ ] [AP51.7: Server dispatch, features and stdio](07-server-runtime-and-stdio.md)
- [ ] [AP51.8: HTTP server and authorization](08-server-http-and-auth.md)
- [ ] [AP51.9: Interoperability, documentation and acceptance](09-acceptance.md)

This is the only AP51 work-package checklist. AP51.1 precedes all implementation;
AP51.2 precedes AP51.3; AP51.4 and AP51.5 use that runtime. AP51.7 builds on
AP51.2/AP51.3 and the approved interaction contract; AP51.8 builds on AP51.7.
AP51.6 uses the selected transports/features and, for server-side extensions,
AP51.7/AP51.8. AP51.9 accepts the complete client and server result. Package
numbering does not override these dependencies.

## Acceptance

An FPAS client can use the agreed transports against an independently implemented
MCP server, and an FPAS server can serve an independently implemented MCP client.
Both exercise every advertised capability, handle multi-round interactions
explicitly and shut down without retained process slots, pending rights,
listeners or live native resources. Tests distinguish JSON-RPC errors, tool-level
errors, transport failures, cancellation and timeouts. Concurrent operations and
unrelated ACP clients continue to progress. Repeated connect/fail/close and
continuation/subscription cycles return to the allocation/resource baseline.

The delivered handbook, generated API, CLI/editor signatures and examples must
agree. Unsupported capabilities and versions fail explicitly. All relevant
[shared boundaries](../shared-constraints.md) and
[required checks](../development-process.md#required-checks) pass. Deferred
features are documented rather than silently counted as implemented.
