# AP51.7: Server dispatch, features and stdio

Package: [AP51](README.md). Status: Not started; blocked on AP51.1–AP51.3 and
Q59–Q61. Server support itself is approved by Q58.

## Contract and scope

Let FPAS applications implement MCP servers through the approved server unit.
Use the pinned SDK's server role and the shared protocol binding. Expose incoming
requests as typed work with single-use response rights; application code runs on
FPAS workers, never on the native SDK executor. The precise event/handler API,
admission limits and scheduling policy are agreed in AP51.1.

Serve discovery and the selected tools, resources, prompts and completion methods.
Advertised capabilities must match registered application behavior. Validate
incoming messages and request metadata before dispatch; distinguish protocol
errors, handler failures, tool errors and cancellation. Preserve caller identity
per request, without inventing protocol sessions. Request IDs are scoped to their
transport/request context, not globally unique across clients.

Provide explicit stdin/stdout transport ownership for a launched FPAS server and
bounded message embedding. Protocol output must not mix with `Console.WriteLn`,
diagnostics or debugger output. Specify how competing stdin/stdout users are
rejected or routed before taking ownership; acquisition failure must release
every reservation. Do not silently replace the embedding host's standard streams.
Reserve stdout for protocol framing and use the agreed diagnostic channel.

Implement progress and subscriptions, including ordered updates and cancellation
while a subscriber is slow or silent. Bound request queues, response buffers,
subscriber count and retained application values. Stop admitting work before
shutdown, cancel owned work cooperatively, and release response rights and GC
roots when a request ends. A cancelled or closed responder cannot be revived by
a later request with the same wire ID.

For MRTR, servers explicitly return `input_required` and validate the subsequent
input responses. The retry is a new request and may reach another instance; it
must not depend on a suspended callback or hidden MCP session. Application state
that influences authority or business logic needs integrity protection, bounded
expiry and replay handling under the agreed policy. Determine which guarantees
the SDK provides and which are application responsibilities. No implicit
re-execution of a completed side effect is permitted by the wrapper contract.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/server/` | SDK server adapter, validated dispatch, response rights and notifications |
| Create | `crates/fpas-mcp/src/transport/server_stdio.rs` | Explicit process stdin/stdout ownership and framing |
| Create | `crates/fpas-vm/src/vm/hosted/mcp/server/` | FPAS dispatch, task ownership, response conversion and teardown |
| Modify | `crates/fpas-sema/src/std_registry/loaded/mcp/`, `crates/fpas-bytecode/src/intrinsic/mcp.rs` | Public server types and operations |
| Inspect/modify | `crates/fpas-vm/src/vm/hosted/server/`, `crates/fpas-vm/src/vm/hosted/console.rs` | Reuse cooperative server lifetime and coordinate standard-stream ownership |
| Create | `crates/fpas-mcp/tests/server.rs`, `tests/stdlib/mcp/server_test.fpas`, `tests/fixtures/mcp_server.fpas` | Server dispatch, stream and public API regression cases |
| Create | `docs/pascal/std/ai/mcp/server.md` | Delivered server API, execution policy, streams and MRTR responsibilities |

Inspect CLI/debugger stream owners in AP51.1 and name additional affected paths
before implementation. Preserve explicit process-control authority in `Std.Server`;
starting an MCP server does not grant force-exit or signal-handling authority.

## Acceptance and verification

An independent client discovers and invokes an FPAS stdio server and receives
tools, resources, prompts and selected interactions. Test concurrent requests,
duplicate IDs in distinct client contexts, exactly-once reply admission, forged
responders, malformed input, handler failure, cancellation and close/reopen.
Verify protocol stdout remains clean and conflicting stream ownership fails
before effects. Exercise multiple MRTR rounds, tampered/expired/replayed state
under the selected policy, bounded subscriptions and disconnects. Repeated
serve/stop cycles release streams, tasks, closures, payloads and native handles.
