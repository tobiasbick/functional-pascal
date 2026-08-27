# Future: WebSocket Transport

> Deferred. HTTP protocol switching and WebSocket framing are not currently implemented.

WebSocket should be a message transport over the existing TCP/TLS and HTTP foundations. Its
interface should hide handshake headers, frame fragmentation, masking, control frames, and close
state while preserving explicit limits and cancellation.

## Proposed scope

- Client connections for `ws` and verified `wss` URLs.
- Server upgrade validation that transfers an accepted HTTP connection to a WebSocket session.
- Text and binary messages assembled across fragmented frames under a caller-selected size limit.
- Ping, pong, and close handling with validated status codes and UTF-8 reasons.
- Read/write deadlines, cancellation, bounded outbound queues, and observable backpressure.
- Correct client masking, server rejection of masked-direction violations, and cryptographic
  handshake validation.

## Ownership

One session owns the underlying connection after a successful upgrade. The HTTP server must not
write a normal response or close that connection afterward. Concurrent sends may be serialized by
the module; concurrent receives should be rejected unless the interface explicitly owns a single
reader task.

## Excluded from the first slice

- Compression extensions.
- Browser-specific client facilities.
- Automatic reconnection or application-level subscriptions.
- A distributed messaging protocol layered into the transport module.

## Acceptance requirements

- Client and server interoperate with independent RFC 6455 implementations over plain and TLS
  connections.
- Fragmented messages, split reads, control-frame interleaving, and partial writes work correctly.
- Invalid handshakes, masks, opcodes, lengths, UTF-8, and close frames are rejected with bounded
  resource use.
- Slow peers apply backpressure instead of growing memory without limit.
- Cancellation and close wake every blocked operation and release the connection exactly once.
