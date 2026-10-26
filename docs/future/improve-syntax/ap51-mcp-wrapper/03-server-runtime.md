# AP51.3: Owned runtime and FPAS handler dispatch

Package: [AP51](README.md). Status: Complete.

## Contract

Run the official SDK server on a native executor. Detach validated requests
into a bounded queue for the owning FPAS serving task. The library invokes
registered handlers and returns their outcomes through the SDK. Callers do not
write the MCP protocol loop; SDK executor tasks do not execute FPAS callbacks.

Retain functions and captures through the VM's GC. Follow FPAS task ownership,
cancellation and debugger rules. Reject incompatible reentry or concurrent
serving as specified in AP51.1. Applications can forward bounded requests to
another task that owns their state and await its result. Cancelled queued work
must not start an abandoned effect.

Reuse the VM's task-owned callback continuations and root traversal for handler
frames. MCP serving must preserve callback suspension and ordinary debugger
stops through that execution path.

The MCP registry owns server identities, registrations, active calls and
one-use responders. Distinguish the server's application lifetime from each
ACP attachment's lifetime. An attachment cannot use another attachment's
authority or reply rights. Reusing wire IDs, session labels or server names
never revives terminal rights.

Reserve capacity before dispatch and effects. Bound SDK pending work, bodies,
catalog bytes, handlers, responders and retained result data. Keep cancellation,
close and terminal outcomes available when ordinary admission is full.
Shutdown stops admission, cancels cooperative work and releases native ownership
under one deadline. Report cleanup failures.

Define reply/cancel ordering and the terminal point of a handler result.
Disconnect can leave a remote caller uncertain about an applied effect.
Do not replay it automatically or claim that cancellation reverses it.
Applications own any action identity and deduplication they require.

`crates/fpas-mcp/src/server/` implements native admission, detached calls,
exclusive reader ownership, endpoint retirement and aliased response rights.
`crates/fpas-vm/src/vm/hosted/mcp/` retains callback roots and enters normal
task-owned callback continuations. Cooperative serving supports suspension,
debugger execution and collection maintenance. Native cleanup receipts join
opening listeners and accepted sockets independently of caller-owned servers.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/server/` | SDK adapter, bounded admission and response ownership |
| Create | `crates/fpas-vm/src/vm/hosted/mcp/` | Registry, callback retention and cooperative dispatch |
| Modify | `crates/fpas-vm/src/vm/hosted/mod.rs`, VM root/teardown owners | Dispatch, GC roots and common cleanup |
| Inspect/reuse | `crates/fpas-vm/src/vm/hosted/acp/`, `crates/fpas-vm/src/vm/hosted/server/` | Borrowed/owned resources and shutdown deadlines |
| Reuse | `crates/fpas-vm/src/vm/hosted/callbacks/` | Resumable callback invocation and GC rooting |
| Create | `crates/fpas-mcp/tests/server.rs`, `crates/fpas-mcp/tests/runtime.rs`, `tests/stdlib/mcp/handler_test.fpas` | Observable callbacks, runtime and lifetime behavior |
| Create/modify | `crates/fpas-cli/tests/batch_debug/mcp.rs`, `tests/fixtures/mcp_debug_server.fpas` | Capture retention, suspension and task ownership |
| Create | `docs/pascal/std/ai/mcp/README.md` | Delivered serving and lifetime contract |

## Acceptance

Cover handlers on the designated FPAS task, retained captures, cooperative
waits and normal/debugger hosts. Fixtures cover concurrent callers, malformed
input, handler failures, duplicate/late replies, cancellation, full queues and
shutdown. Cancelled queued calls perform no effects. Repeat
register/serve/fail/close cycles and verify native slots and GC payloads return
to baseline. Unrelated ACP traffic keeps progressing.
