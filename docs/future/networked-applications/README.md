# Future: Networked Application Platform

> Deferred. This is a capability roadmap, not current FPAS behavior.

Functional Pascal already provides blocking TCP/TLS connections and listeners, HTTP client and
server helpers, tasks, JSON, filesystem access, and a local terminal UI. Long-running secure
networked applications still need durable transactions, security primitives, coordination between
background work and interactive hosts, production lifecycle support, and explicit distributed-node
semantics.

This plan keeps those concerns in separate modules. The first approved language extension is the
implemented `channel of T` type; any additional syntax or semantic change still requires its own
explicit decision.

## Capability plans

| Concern | Plan | Intended seam |
|---------|------|---------------|
| Durable storage | [SQLite](sqlite.md) | One concrete embedded database adapter with opaque handles and transactions |
| Security | [Cryptography](cryptography.md) | High-level safe operations over operating-system randomness and reviewed algorithms |
| Coordination | [Concurrency](concurrency.md) | Bounded channels, cancellation, multi-wait, and supervised task ownership |
| Bidirectional transport | [WebSocket](websocket.md) | Bounded message transport over existing TCP/TLS and HTTP foundations |
| Datagrams | [UDP](udp.md) | Explicit datagram endpoints without reliability promises |
| Interactive clients | [TUI background events](tui-background-events.md) | Wakeable application-owned event delivery onto the main TUI thread |
| Process lifetime | [Server lifecycle](server-lifecycle.md) | Coordinated stop signals, listener closure, draining, and deadlines |
| Operations | [Observability](observability.md) | Structured events, metrics, and health state independent of presentation |
| Multiple nodes | [Distributed nodes](distributed-nodes.md) | Authenticated envelopes, durable delivery, synchronization, and conflict policy |

## Dependency order

1. Settle concurrency ownership and cryptographic requirements before exposing higher-level
   background or distributed interfaces.
2. Add SQLite as a concrete adapter and server lifecycle support so state and work can shut down
   consistently.
3. Add WebSocket and TUI background-event integration over the settled cancellation and queueing
   model.
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
