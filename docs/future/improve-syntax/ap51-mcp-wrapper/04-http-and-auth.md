# AP51.4: Streamable HTTP and authentication boundary

Package: [AP51](README.md). Status: Not started; blocked on AP51.1–AP51.3,
especially Q60 and Q62.

## Contract and scope

Implement the selected modern Streamable HTTP client using the SDK transport.
Each request uses POST and accepts both JSON and request-scoped SSE, including
progress and subscription streams. Generate required protocol, method, name and
applicable parameter-routing headers from the same request representation as
the body. Apply specified encoding and omission rules rather than interpolating
untrusted strings into headers.

Do not use legacy session headers, a GET event channel, or Last-Event-ID resume.
Transport cancellation follows the selected specification: closing the HTTP
response stream differs from stdio's cancellation notification. A dropped
connection is not proof a tool invocation did not execute; do not automatically
replay uncertain operations. Distinguish explicit MRTR continuation from retry.

Bound response bodies, SSE frames, event queues, active streams and quiet waits.
Support split UTF-8, SSE comments, JSON error bodies, HTTP errors, disconnects
and partial results. Reconnects must not restore consumed local request rights.

The proposed authentication contract accepts caller-provided credentials and
reports authentication-required/denied results without opening a browser or
persisting tokens. Bind credentials to the intended destination and avoid
forwarding authorization on cross-origin redirects. Preserve TLS validation and
explicit caller transport policy. If Q62 selects managed OAuth, expand this work
before implementation to cover issuer/resource validation, CIMD/registration,
PKCE/callbacks, refresh, cancellation and token storage; do not claim OAuth support
from a bearer-token field alone.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/transport/http.rs`, `crates/fpas-mcp/src/auth.rs` | HTTP transport configuration and selected authentication contract |
| Modify | `crates/fpas-mcp/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | Minimal SDK HTTP/TLS/auth features |
| Modify | `crates/fpas-vm/src/vm/hosted/mcp/`, `crates/fpas-sema/src/std_registry/loaded/mcp/` | FPAS options, results and cooperative cancellation |
| Create | `crates/fpas-mcp/tests/http.rs`, `tests/stdlib/mcp/http_test.fpas` | Deterministic loopback HTTP/SSE and public API cases |
| Create | `docs/pascal/std/ai/mcp/transports.md`, `docs/pascal/std/ai/mcp/authentication.md` | Actual supported transport and auth behavior |

## Acceptance and verification

Use a local independent HTTP peer to verify required headers, both response
formats, auth challenges, redirects, cancellation, bounded streams and clean
shutdown. Test that uncertain disconnects do not invoke a tool twice and that
credentials do not cross destination boundaries or appear in default diagnostics.
Verify all adopted SDK HTTP defaults against the approved revision. Cover both
unauthed loopback use and the selected authenticated path without external
accounts or live credentials in mandatory tests.
