# `Std.Net`

Hosted blocking TCP/TLS listeners and connections with explicit timeouts and byte arrays.

```pascal
uses Std.Net;

case Connect('127.0.0.1', 8080, 5000) of
  Ok(Connection):
  begin
    // Use Read, Write, SetTimeout, and Close.
  end;
  Error(Message): panic(Message)
end
```

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Connection` | opaque VM-owned TCP or TLS connection |
| type | `Listener` | opaque VM-owned TCP or TLS listener |
| function | `Connect(Host: string; Port: integer; TimeoutMillis: integer): Result of Connection, string` | resolves and connects |
| function | `ConnectTls(Host: string; Port: integer; TimeoutMillis: integer): Result of Connection, string` | resolves, connects, and completes a verified TLS handshake |
| function | `ConnectWithCancellation(Host: string; Port: integer; TimeoutMillis: integer; Token: Std.Task.CancellationToken): Result of Connection, string` | cancellable TCP attempts; OS DNS checked on return |
| function | `ConnectTlsWithCancellation(Host: string; Port: integer; TimeoutMillis: integer; Token: Std.Task.CancellationToken): Result of Connection, string` | also observes cancellation during TLS handshake I/O |
| function | `Listen(Host: string; Port: integer): Result of Listener, string` | binds one TCP listener |
| function | `ListenTls(Host: string; Port: integer; CertificatePath: string; PrivateKeyPath: string; HandshakeTimeoutMillis: integer): Result of Listener, string` | loads PEM credentials and binds one TLS listener |
| function | `Accept(Listener): Result of Connection, string` | blocks until one client connects |
| function | `AcceptWithCancellation(Listener; Token: Std.Task.CancellationToken): Result of Connection, string` | blocks until one client connects or cancellation is requested |
| function | `CloseListener(Listener): Result of boolean, string` | invalidates the listener handle |
| function | `SetTimeout(Connection; TimeoutMillis: integer): Result of boolean, string` | sets read/write timeout; zero disables it |
| function | `Read(Connection; MaxBytes: integer): Result of array of integer, string` | empty array means EOF |
| function | `ReadWithCancellation(Connection; MaxBytes: integer; Token: Std.Task.CancellationToken): Result of array of integer, string` | reads a chunk or reports cancellation; leaves the connection open |
| function | `Write(Connection; Data: array of integer): Result of integer, string` | returns bytes written; partial writes are possible |
| function | `WriteWithCancellation(Connection; Data: array of integer; Token: Std.Task.CancellationToken): Result of integer, string` | returns accepted bytes or reports cancellation before progress |
| function | `Close(Connection): Result of boolean, string` | invalidates the handle |

`ConnectTls` verifies the server certificate and requested hostname through the operating system's
trust policy. It does not expose an insecure certificate bypass. The returned `Connection` uses the
same `Read`, `Write`, `SetTimeout`, and `Close` functions as a plain TCP connection.

The connect timeout bounds TCP establishment and, for `ConnectTls`, the TLS handshake. Call
`SetTimeout` to configure subsequent byte reads and writes.

`Listen` binds the requested local host and port. `ListenTls` first loads a PEM certificate chain and
PEM private key, then binds the requested address. Its positive handshake timeout is limited to five
minutes. Client certificates and multiple certificates selected through SNI are not supported.

`Accept` and `AcceptWithCancellation` return the same `Connection` type for TCP and TLS listeners,
so `SetTimeout`, `Read`, `Write`, and `Close` apply to both. A TLS listener completes the handshake
before returning and discards failed or timed-out handshakes while waiting for a valid client.
Listener handles are closed separately with `CloseListener`. Both accept functions and connection
I/O block the VM worker that executes them.

`AcceptWithCancellation` observes a `Std.Task.CancellationToken`. If cancellation wins before a
connection is returned, the function returns `Error('Network accept cancelled')`. It leaves the
listener open, so another task may accept from it later. `Accept` remains available when the caller
does not need application-controlled cancellation.

Byte values must be in `0..255`. A single `Read` or `Write` is limited to 1 MiB. Timeouts are limited
to `300000` milliseconds; `Connect` and `ConnectTls` require a positive timeout. Client and listener
ports must be in `1..65535`.

Calls block their VM worker thread. Connections and listeners belong to one VM, are shared safely
with its tasks, and are released when that VM ends. `Close` interrupts a blocked `Read` or `Write`
on the connection, and `CloseListener` interrupts an active `Accept`, including a pending TLS
handshake. VM cancellation interrupts established connection I/O and listener waits before joining
task workers. The cancellation-aware connect variants also interrupt pending TCP attempts; OS
resolution and synchronous TLS verification have the limits described below.
Closed handles cannot be reused.

FPAS code runs with the host process's network permissions. `Std.Net` does not sandbox destinations.

## `ConnectWithCancellation` and `ConnectTlsWithCancellation`

These variants take a `Std.Task.CancellationToken` and use the same host, port, and positive
`1..300000` ms timeout limits as `Connect` and `ConnectTls`. TLS uses the same operating-system trust
policy; cancellation does not bypass certificate or hostname verification.

- A token already cancelled prevents DNS resolution and socket creation for valid arguments.
- Cancellation is checked before and after DNS resolution, while each TCP attempt is pending,
  during TLS handshake I/O, and before publishing a handle. Observed cancellation returns
  `Error('Network connect cancelled')`. Pending sockets are dropped without publishing a handle.
- The new variants use one monotonic budget starting before resolution, shared by all resolved
  addresses and the TLS handshake. Exhaustion observed at a checkpoint returns
  `Error('Network connect timed out')`. A successful final checkpoint wins over cancellation arriving
  immediately afterward. Existing `Connect` and `ConnectTls` retain their timeout behavior.
- Each address uses one non-blocking TCP attempt. Pending attempts check cancellation and the
  shared deadline between socket readiness checks, parking for at most 10 ms between checks.
  Cancellation or expiry closes that attempt's socket; polling does not restart the connection.
- **Cancellation is not a hard wall-clock bound:** the operating-system DNS call cannot be
  interrupted. TLS platform configuration and certificate verification also run synchronously.
  Time spent there counts against the budget, but cancellation and expiry
  may only be reported after the OS call returns. TLS socket waits check cancellation between
  handshake I/O steps. VM shutdown has the same OS-call limitation.
- Resolution, connection, and TLS failures remain errors rather than successful handles. If
  cancellation is observed at the same checkpoint as an OS error or expired budget, cancellation
  takes precedence. Successful connections initially have no read/write timeout; use `SetTimeout`.

## `ReadWithCancellation`

Reads at most `MaxBytes` from an established TCP or TLS connection. The same byte-size limits and
EOF representation as `Read` apply. Import `Std.Task` to create the cancellation source and token.

- Cancellation observed before a read attempt returns `Error('Network read cancelled')` without
  closing the connection. A token that is already cancelled does not consume available bytes.
- A successful read wins over cancellation arriving during that read; its bytes are returned.
- The read timeout configured by `SetTimeout` bounds the read phase with one monotonic deadline;
  expiry returns `Error('Network read timed out')`. Zero disables that deadline, not cancellation.
- Operations on one connection are serialized. Cancellation is also checked while waiting for
  another operation to release the connection, but the read timeout starts after that wait.
- Cancellation and timeout preserve TLS state and the configured socket timeouts. Later reads may
  continue with a fresh token. Cancellation is polled at intervals of up to 10 ms between I/O
  attempts; OS scheduling can add latency. The call still occupies its VM worker.

## `WriteWithCancellation`

Writes one bounded chunk to an established TCP or TLS connection. Byte values and the 1 MiB limit
are the same as for `Write`. Import `Std.Task` to create the cancellation source and token.

- `Ok(N)` means the first `N` bytes were accepted by the local transport. Partial writes are allowed;
  retry only the remaining suffix. The function does not loop to send the entire input.
- Accepted bytes take priority over cancellation or timeout arriving during a write attempt. A
  positive count is never replaced by a cancellation error after those bytes have been accepted.
- Cancellation before an attempt returns `Error('Network write cancelled')`. No bytes from this
  call's input have been accepted. The connection stays open; previously accepted data is not undone.
- The write timeout from `SetTimeout` starts after acquiring the connection lock and uses one
  monotonic deadline across retries. Expiry before progress returns `Error('Network write timed out')`.
  Zero disables the deadline, not cancellation. Lock waits also observe cancellation.
- For TLS, accepted bytes can remain buffered locally. `Ok(N)` is not confirmation of delivery to
  the peer. A later operation may advance buffered output even if that later call is cancelled
  before accepting its own input. Never resend a prefix already reported as accepted.
- An empty input may return `Ok(0)`; a pre-cancelled token still returns cancellation. As with
  `Write`, a zero count does not establish that the connection has closed.

Both cancellable I/O functions preserve socket timeout settings and normally restore blocking mode
before returning. If restoring that mode fails, the connection is invalidated; a successful byte
count or read result is still returned rather than losing progress information. Cancellation is
polled between I/O attempts with the same 10 ms interval as `ReadWithCancellation`; the call occupies
its VM worker.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| VM connection registry | [`connections.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections.rs) |
| Cancellable connection establishment | [`establishment.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections/establishment.rs) |
| Cancellable TCP attempts | [`establishment/tcp.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections/establishment/tcp.rs) |
| Cancellable reads | [`cancellable_read.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections/cancellable_read.rs) |
| Cancellable writes | [`cancellable_write.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections/cancellable_write.rs) |
| Cancellation and deadline polling | [`polling.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections/polling.rs) |
| VM listener registry | [`listeners.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/listeners.rs) |
| Cancellation registry | [`cancellation/registry.rs`](../../../../crates/fpas-vm/src/vm/cancellation/registry.rs) |
| TCP/TLS transport | [`transport.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/transport.rs) |
| TLS client verification | [`client.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/tls/client.rs) |
| TLS server credentials and handshake | [`server.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/tls/server.rs) |
| Hosted dispatch | [`net/mod.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/mod.rs) |
| Registration | [`loaded/net.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/net.rs) |
| Intrinsic ids | [`intrinsic/net.rs`](../../../../crates/fpas-bytecode/src/intrinsic/net.rs) |

## See also

- [Networking index](README.md)
- [`Std.Http`](http.md)
- [`Std.Task`](../concurrency/task.md)
- [Standard library index](../README.md)
