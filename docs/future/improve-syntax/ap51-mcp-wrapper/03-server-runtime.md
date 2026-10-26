# AP51.3: Owned server runtime and dispatch

Package: [AP51](README.md). Status: Not started; depends on AP51.1 and AP51.2.

## Contract

Run the official SDK server on a native executor and detach typed request data
into a bounded FPAS receive/respond path. Application handlers run on FPAS
workers. A one-use responder grants a reply right, not blanket authority to
invoke tools or access host resources.

Own server/endpoint handles, active requests, payloads and cancellation within
the VM. Apply existing native-handle failure rules. Define reply/cancel ordering;
wire-ID or session/endpoint reuse never revives terminal rights.

Reserve capacity before effects. Bound SDK-side pending work and retained bytes,
not only the application queue. Keep shutdown/cancellation and terminal outcomes
available when ordinary admission is full. Stop admission, cancel cooperative
work and release native ownership under one shutdown budget.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/server/` | SDK adapter and response ownership |
| Create | `crates/fpas-vm/src/vm/hosted/mcp/` | Registry, conversion, cooperative waits and teardown |
| Modify | `crates/fpas-vm/src/vm/hosted/mod.rs`, actual VM lifetime owners | Dispatch and common resource cleanup |
| Inspect/reuse | `crates/fpas-vm/src/vm/hosted/server/` | Shared shutdown deadline/authority rules |
| Create | `crates/fpas-mcp/tests/server.rs`, `tests/stdlib/mcp/server_test.fpas` | Runtime and public API tests |
| Create | `docs/pascal/std/ai/mcp/server.md` | Delivered execution/lifetime contract |

## Acceptance

Local fixtures cover concurrency, malformed input, failures, duplicate/late
replies, cancellation, full queues and quiet shutdown. Repeat create/fail/close
cycles and check native slots and GC payload baselines. Preserve unrelated ACP
progress. These tests need no live Codex access and do not replace AP51.6's
home acceptance.
