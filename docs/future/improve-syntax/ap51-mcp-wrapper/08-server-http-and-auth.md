# AP51.8: HTTP server and authorization

Package: [AP51](README.md). Status: Not started; blocked on AP51.1, AP51.7 and
Q59–Q62. Use AP51.4's shared HTTP contract; an FPAS client is not the sole test peer.

## Contract and scope

Expose the approved MCP server through a Streamable HTTP endpoint backed by the
SDK and owned by the FPAS server lifetime. Bind addresses, ports and TLS policy
explicitly. Account for listener, request, body, stream and handler capacity
before admitting application work. Reuse existing native networking/lifecycle
facilities where they fit; choose any adapter to SDK HTTP listener ownership in
AP51.1 rather than silently maintaining two unrelated shutdown paths.

Validate protocol version and required method/name/parameter-routing headers
against the decoded body before dispatch. Handle JSON and request-scoped SSE,
progress and `subscriptions/listen` using the modern protocol. No legacy GET
stream or protocol session is required. Stream termination cancels associated
work according to the protocol and application cancellation contract; it does
not prove that an already admitted side effect was undone.

Q62 defines authentication and authorization, not just a client token parameter.
The proposed server boundary uses explicitly configured credential validation
and per-request authorization before application effects. Define identity,
issuer/resource/audience and scope checks for the selected credential model,
auth challenges and protected-resource metadata where required. Any application
validation code executes on FPAS workers under the same limits and cancellation
rules as other work. A header's presence or an upstream claim is not successful
validation. Trusted reverse-proxy metadata requires an explicit trust boundary.

Allow an explicitly selected unauthenticated local deployment without claiming
OAuth support. Remote exposure, origin/host checks, TLS/proxy termination and
credential policy must be documented choices. An MCP resource server is not an
OAuth authorization server; implementing the latter is not implied by Q58.
Keep authenticated callers, subscriptions and MRTR state isolated. Cache hints
must not permit private results to cross caller identities.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/server/http.rs`, `crates/fpas-mcp/src/server/authorization.rs` | HTTP serving and explicit authorization boundary |
| Modify | `crates/fpas-mcp/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | Selected SDK server/HTTP/TLS features |
| Modify | `crates/fpas-vm/src/vm/hosted/mcp/server/`, `crates/fpas-sema/src/std_registry/loaded/mcp/` | FPAS listener/configuration APIs and lifecycle integration |
| Inspect/modify | `crates/fpas-vm/src/vm/hosted/net/listeners.rs`, `crates/fpas-vm/src/vm/hosted/server/` | Explicit listener adapter/ownership and common shutdown deadline |
| Create | `crates/fpas-mcp/tests/server_http.rs`, `tests/stdlib/mcp/server_http_test.fpas` | Independent HTTP client, authorization and concurrent lifecycle cases |
| Modify | `docs/pascal/std/ai/mcp/server.md`, `docs/pascal/std/ai/mcp/authentication.md`, `docs/pascal/std/ai/mcp/transports.md` | Actual server transport and deployment contract |

## Acceptance and verification

An independent HTTP client uses an FPAS server through discovery, a tool call,
an MRTR interaction and a subscription. Test mismatched/missing routing headers,
malformed bodies, rejected credentials/scopes, caller isolation and authorization
before observable handler effects. Use local fixtures for all mandatory checks.
Verify slow readers, quiet streams, handler failure, disconnects and full queues
respect bounds and do not block unrelated clients. Failed bind/admission and
repeated start/stop return listener and request capacity. Shutdown shares one
deadline, releases native resources and leaves no registered or live handlers.
