# AP50.3: Native connection driver and transports

Package: [AP50](README.md). Status: planned; implementation not started.

## Prerequisites

[AP50.1](01-schema-and-types.md) supplies types/codec.
[AP50.2](02-process-support.md) supplies precise subprocess status and common
ownership. The message transport can be implemented before its subprocess
adapter, but this work package is delivered only when both adapters work.

## Contract and architecture

Implement [connection setup](interface.md#connection-setup), event ownership,
ordering, bounds and shutdown. `fpas-acp` owns SDK connection orchestration;
`fpas-vm` owns handles, task suspension, process access and lifetime.

Run one native driver executor per VM with a current-thread Tokio runtime.
Each connection is an independently driven SDK task. The VM exchanges owned
commands/events through bounded native queues. Reuse the existing Tokio
version and add its required runtime/sync/time/I/O features on all hosted
platforms; do not make Unix functionality depend on a Windows-only dependency.
Use SDK generic transport adapters and the futures/Tokio compatibility adapter
where necessary. Do not expose this executor as a new FPAS language scheduler.

The process transport is an adapter over existing VM-owned process supervision.
Refactor the process core into focused shared native modules if visibility is
insufficient. It owns the child exactly once. Do not launch a second child via
SDK `AcpAgent`, unwrap foreign public handles, or copy process-group cleanup.
Nonblocking pipe probes yield through the driver timer/readiness path, never a
busy loop or a sleeping FPAS worker.

SDK ordered callbacks copy/detach the event and arrange response work outside
the dispatch loop. They never await user interaction or peer-dependent calls
inside that loop. Install the incoming-request cancellation handler before
dispatching the request to the application. A responder retains native SDK
response state, not a borrowed stack frame or FPAS closure.

## Lifecycle and failure rules

Connection states are `Uninitialized`, `Initializing`, `Ready`, `Closing`,
`Closed` and `Failed`. AP50.4 adds the initialization/session transitions.
`Closing`, `Closed` and `Failed` reject new sends and host effects. A failed
driver retains enough bounded terminal state to settle every admitted request.

`TryReceive` consumes one event. `Receive`, `Close` and `Release` use cooperative VM
suspension and monotonic deadlines, including the ordinary debugger execution
lane. Validation has priority, then an already requested cancellation, then a
ready result, then timeout, matching existing controlled waits. Waking a task
cannot reuse a consumed event or retain an unregistered VM root.

Native threads store Rust-owned detached data only. FPAS values are constructed
on a VM execution lane after dequeue, using the current execution-domain/root
publication contract. Do not hold process, connection or collector locks while
calling back into another owner. Closing wakes all waiting tasks and joins the
driver after its retained responders and transport workers have been resolved.

Implement the reserved failure/close path and byte budgets in
[interface.md](interface.md#defaults-and-bounds). Reject a full outbound queue
before publication. A full inbound queue fails visibly and remains closeable.
Keep draining stderr within its separate bound even if the application is slow.
Fragmented UTF-8, partial writes and an unterminated overlong line must not
corrupt another frame or cause unbounded buffering. EOF with an incomplete
nonempty JSON frame is `InvalidMessage`, not successful clean completion.

No reconnect/retry is automatic. A peer's clean exit is a connection termination
with precise status; requests without replies receive connection failures.
Repeated `Close` returns false after the first successful close. `Release`
discards retained events and removes the handle, returning its slot; subsequent
alias use is a closed-handle error. Session close does not close the transport.
VM completion, panic and debugger disconnect terminate/reap all owned
descendants and clear all native registries, including unreleased closed handles.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-acp/src/connection/`, `src/transport/` | SDK driver, deferred responders, message and byte adapters |
| Create | `crates/fpas-acp/tests/transport.rs`, `tests/connection.rs` | Framing, raw extension routing, ordered dispatch and bounded failure |
| Modify | `crates/fpas-vm/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | Adapter dependency and cross-platform Tokio/compatibility features |
| Create | `crates/fpas-vm/src/vm/hosted/acp/mod.rs`, `registry.rs`, `dispatch.rs`, `transport.rs` | VM ownership, intrinsic bridge, process/message endpoint lifecycle |
| Modify | `crates/fpas-vm/src/vm/hosted/mod.rs` | Hosted state creation and shutdown integration |
| Modify | `crates/fpas-vm/src/vm/hosted/proc/` | Reuse process ownership through an internal production adapter |
| Modify | `crates/fpas-vm/src/vm/tasks/suspension.rs`, `tasks/pool/suspension.rs`, `tasks/scheduler/` | Cooperative receive/close and wakeup ownership |
| Modify | `crates/fpas-vm/src/vm/debug/`, `crates/fpas-vm/src/vm/domain/EXECUTION.md` | Debugger waits, host-effect policy and root ownership documentation |
| Create/modify | `crates/fpas-bytecode/src/intrinsic/acp.rs`, intrinsic metadata and `crates/fpas-compiler/src/intrinsic_catalog.rs` | Correct ACP intrinsic routing and hosted execution ownership |
| Create | `crates/fpas-vm/src/vm/hosted/acp/tests/` | Handle aliases, cancellation, task progress and VM cleanup |
| Create | `docs/pascal/std/ai/acp/connections.md` | Delivered connection, transport, queue and shutdown contract |

Follow the current generic intrinsic serialization/execution-owner paths.
Regenerate native editor declarations after completing operation registration.
Do not postpone a catalog, artifact or debugger update to AP50.7.

## Acceptance and verification

- A real process peer sends split JSON/UTF-8 frames, floods stderr and requires
  partial stdin writes. No data stream contaminates another.
- The production message endpoint works in both directions, preserves frame
  ownership, reports full queues without acceptance, and processes EOF once.
- A delayed inbound request does not block another session's update, a second
  response, cancellation or close. Use a peer that waits for this behavior.
- With one FPAS worker, an idle receive or closing connection does not prevent
  an unrelated task from progressing. Waiting cancellation/timeout consumes
  nothing and leaves protocol work active.
- Exhaust every configured entry/byte bound at the boundary and one over it.
  Failure remains observable even when the ordinary event queue is full.
- Alias/foreign/closed handle use fails before native effects. Normal exit,
  malformed EOF, process crash, driver failure, VM runtime failure and debugger
  disconnect settle requests once and release children/threads/registries.
- Repeat near-threshold close/cancellation races without timing-based sleeps
  as the assertion mechanism. Coordinate through actual messages and receipts.

Run owning Rust VM/adapter tests and required workspace checks. Investigate any
observed FPAS responsiveness/performance issue with `fpas run --profile` before
adding a sampler or benchmark variant. Record a forced dogfooding fallback in
the existing gaps file with a minimal reproducer.
