# Future: UDP Sockets

> Deferred. No implementation is scheduled without a concrete datagram requirement.

UDP provides message-oriented delivery without connections, ordering, retry, congestion handling,
or duplicate suppression. The module must expose those limitations instead of resembling the
reliable `Std.Net.Connection` interface.

## Proposed scope

- Opaque UDP socket and endpoint values with IPv4 and IPv6 support.
- Bind, connect-to-default-peer, send, send-to, receive, receive-from, timeout, and close operations.
- Explicit maximum datagram sizes and truncation reporting.
- Broadcast enablement and selected multicast operations only when required by a real discovery
  protocol.
- Cancellation and VM-shutdown cleanup consistent with other hosted network resources.

## Non-goals

- Reliability, retransmission, ordering, sessions, or stream emulation.
- Automatic peer discovery.
- QUIC, DTLS, or a custom encrypted datagram protocol in the first slice.
- Using UDP as a prerequisite for ordinary node synchronization over HTTPS or WebSocket.

## Acceptance requirements

- Loopback send/receive works for IPv4 and IPv6 where the host supports them.
- Empty, maximum-sized, oversized, truncated, timed-out, and canceled receives are distinguished.
- Source endpoints are preserved and invalid addresses or ports produce actionable errors.
- Closing a socket interrupts blocked operations and releases the port deterministically.
