# Future: Distributed Nodes

> Deferred. This plan starts only after storage, cryptography, concurrency, and lifecycle foundations
> exist.

Connecting independent nodes is not only a transport problem. Each node needs durable identity,
authenticated messages, version negotiation, retry-safe delivery, synchronization cursors, and an
explicit conflict policy. WebSocket or HTTPS can carry the protocol; UDP is optional and must not be
assumed.

## Node identity and trust

- Each node owns a persistent identifier and Ed25519 key pair stored outside committed source.
- Peer configuration records the expected public key or follows an explicitly approved trust-on-first-use
  or authority model.
- Signed envelopes include sender, recipient or audience, message identifier, protocol version,
  creation time, expiry, and payload digest.
- Key rotation, revocation, clock skew, replay windows, and compromised-peer removal are part of the
  protocol design.

## Durable delivery

- An SQLite outbox records outbound messages in the same transaction as the local state change that
  created them.
- An inbox records accepted message identifiers before applying effects, making retries idempotent.
- Delivery defines acknowledgements, retry limits, exponential backoff, dead-letter handling, and
  operator-visible failure state.
- Queue and payload limits protect a node from unavailable or hostile peers.

## Synchronization

- Peers negotiate protocol and schema versions before exchanging application data.
- Incremental synchronization uses durable cursors and bounded pages rather than unbounded snapshots.
- Ordering guarantees are stated per stream; a global total order is not assumed.
- Conflict behavior is selected for each data class: immutable events, last-writer rules with a
  defined clock, authoritative ownership, or explicit manual resolution.
- Deletion and retention use durable tombstones or another specified convergence mechanism.

## Discovery and transport adapters

Manual peer configuration and authenticated HTTPS are the first transport adapter. WebSocket may
provide an efficient persistent adapter after its lifecycle and backpressure semantics are stable.
DNS-based or UDP discovery remains optional and cannot establish trust by itself.

## Acceptance requirements

- Two independently started processes authenticate, negotiate versions, and synchronize through
  the public protocol.
- Retries, duplicate delivery, reordering, interruption, restart, and temporary partition converge
  without duplicate effects.
- Invalid signatures, expired envelopes, replayed identifiers, incompatible versions, and revoked
  peers are rejected and observable.
- Outbox and inbox state survives process failure at every transaction boundary.
- Backlogs remain bounded and one failing peer cannot prevent healthy peers from progressing.
- Conflict and deletion behavior is covered by shared protocol scenarios, not adapter-specific
  assumptions.
