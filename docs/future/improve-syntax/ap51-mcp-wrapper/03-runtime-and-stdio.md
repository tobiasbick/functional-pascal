# AP51.3: Owned runtime, message transport and stdio

Package: [AP51](README.md). Status: Not started; blocked on AP51.1 and AP51.2.

## Contract and scope

Provide the approved client runtime, explicit modern discovery, asynchronous
request/result events, cooperative waits and local close/release. A connection
is a VM-owned transport lease, not an MCP session. SDK executor tasks exchange
detached values; FPAS code executes on FPAS workers only.

Support a bounded in-memory message transport and owned child-process stdio.
Reserve connection, retained client metadata, queue and process capacity before
launch. Validate executable/arguments without shell interpolation. Frame stdout
as MCP messages, drain bounded stderr separately, and handle fragmented UTF-8,
partial reads/writes, malformed JSON, EOF and child exit.

Define one terminal outcome per request, cancellation versus result ordering,
deadline expiry, closed-handle alias behavior and foreign-VM rejection. Native
resources must be terminated, reaped and released on every failure path, including
failed connect and abandoned transport construction. Charge retained buffers,
requests, events and continuations to explicit limits rather than just counting
outer events. Full queues must not prevent cleanup or final failure publication.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/connection/`, `crates/fpas-mcp/src/transport/` | SDK execution, discovery, budgets, request ownership and message transport |
| Create | `crates/fpas-vm/src/vm/hosted/mcp/` | VM registry, value conversion, operations and cooperative waits |
| Create | `crates/fpas-vm/src/vm/hosted/proc/mcp.rs` | Adapt the existing owned process registry to the MCP transport |
| Modify | `crates/fpas-vm/src/vm/hosted/mod.rs`, `crates/fpas-vm/src/vm/hosted/proc/mod.rs`, `crates/fpas-vm/Cargo.toml` | Connect runtime dispatch and VM teardown |
| Create | `crates/fpas-mcp/tests/lifecycle.rs`, `tests/fixtures/mcp_peer.fpas`, `tests/stdlib/mcp/connection_test.fpas` | Wire lifecycle and public process/message regression cases |
| Create | `docs/pascal/std/ai/mcp/connections.md` | Limits, effects, ownership and shutdown contract |

Reuse `PipedProcesses` and existing process-group behavior. Extract a shared
transport adapter only where actual common code warrants it; preserve ACP tests
and its distinct protocol lifetime. Expand concrete dispatch/teardown owners
after inspecting their current shape immediately before implementation.

## Acceptance and verification

An independent stdio peer completes modern discovery and a request. Repeated
invalid-admission, spawn-failure and connect/close cycles release all reservations.
Pending requests, subscriptions and continuation aliases cannot revive after
reopening a transport. A slow or failed peer does not block unrelated tasks or
ACP connections. Test cancellation/response races and shutdown under full queues,
bounded output and silent peers. Reuse process descendant cleanup and VM GC
baseline tests; add only missing MCP-specific retention scenarios.
