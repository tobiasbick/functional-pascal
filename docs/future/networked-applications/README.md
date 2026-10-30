# Future: Networked Application Platform

Functional Pascal provides TCP/TLS connections and listeners, HTTP client and server helpers,
tasks, JSON, filesystem access, and a local terminal UI. Long-running secure networked
applications still need durable transactions, security primitives, further transport and lifecycle
adapters, operations tooling, and explicit distributed-node semantics.

This plan keeps those concerns in separate modules. Additional syntax or semantic changes require
their own explicit decisions.

Public names follow the binding [namespace specification](../../specs/std-namespaces.md).
Descriptions of implemented behavior below use canonical names; future
capability plans use their allocated homes. A namespace allocation does not
change a feature's deferred status.

## Implemented foundations

- [Task selection](../../pascal/std/concurrency/task.md#mixed-source-selection),
  [task groups](../../pascal/std/concurrency/task.md#task-groups),
  [supervision](../../pascal/std/concurrency/task.md#startsupervisedtask), and
  [completion waits](../../pascal/std/concurrency/task.md#waiting-for-any-task)
  provide bounded coordination, cancellation and child ownership.
- [Terminal background messages](../../pascal/std/tui/application.md#background-messages-and-owned-work)
  deliver typed events to the UI thread and retain ownership of background work.
- [`Std.Net.Server`](../../pascal/std/network/server.md) owns listeners and task groups,
  coordinates shutdown deadlines, and provides explicit process-level escalation.

## Capability plans

| Concern | Plan | Intended seam |
|---------|------|---------------|
| Durable storage | [SQLite](sqlite.md) | One concrete embedded database adapter with opaque handles and transactions |
| Security | [Cryptography](cryptography.md) | High-level safe operations over operating-system randomness and reviewed algorithms |
| Bidirectional transport | [WebSocket](websocket.md) | Bounded message transport over existing TCP/TLS and HTTP foundations |
| Datagrams | [UDP](udp.md) | Explicit datagram endpoints without reliability promises |
| Process lifetime | [Server lifecycle](server-lifecycle.md) | Coordinated stop signals, listener closure, draining, and deadlines |
| Operations | [Observability](observability.md) | Structured events, metrics, and health state independent of presentation |
| Multiple nodes | [Distributed nodes](distributed-nodes.md) | Authenticated envelopes, durable delivery, synchronization, and conflict policy |

## Dependency order

1. Define cryptographic requirements before exposing higher-level distributed interfaces.
2. Add SQLite as a concrete adapter and extend lifecycle adapters so state and work can shut down
   consistently under the existing ownership and deadline model.
3. Add WebSocket integration using the existing cancellation and queueing model.
4. Add observability at the same seams used for lifecycle, storage, and transport failures.
5. Start distributed-node work only after identity, durable outbox/inbox storage, idempotency, and
   version negotiation have concrete designs.
6. Add UDP only for a selected datagram requirement; it is not a prerequisite for node-to-node
   synchronization.

## Cross-cutting requirements

- Every externally supplied size, duration, queue, frame, statement, and result set must be bounded.
- Blocking host calls must document which task or thread they occupy and how cancellation reaches
  them.
- Opaque resources must have deterministic close behavior and VM-shutdown cleanup.
- Error messages must distinguish invalid input, unavailable peers, timeouts, cancellation, and
  exhausted limits.
- Interfaces should accept dependencies at testable seams instead of constructing global state.
- Protocol parsers require hostile-input, fragmentation, truncation, and limit tests.
- Secrets, credentials, machine paths, and host identity must never appear in committed fixtures or
  reports.

## Completion rule

Each capability remains future work until its implementation, current documentation under
`docs/pascal/`, examples where useful, and focused regression tests land together. Remove or narrow
the corresponding future plan when current documentation fully owns the behavior.
