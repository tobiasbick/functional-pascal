# `Std.Net.Sockets`

Hosted TCP/TLS listeners and connections with explicit timeouts and byte buffers.

```pascal id=example-1 fragment context=header
uses Std.Net.Sockets as Net;
begin
case Net.Connect('127.0.0.1', 8080, 5000) of
  when Ok(const Connection):
    begin
      // Use ReceiveBytes, SendBytes, SetTimeout, and Close.
      null;
    end;
  when Error(const Message):
    panic(Message.Message());
end case;

end.
```

## Operational errors

Every fallible operation returns `NetError`: `Closed(Operation)`,
`Timeout(Cause: HostError)`, `Cancelled(Operation)`, `InvalidArgument(Name, Reason)`,
`ResolveFailed(Cause)`, `ConnectionRefused(Cause)`, `ConnectionLost(Cause)`,
`AddressInUse(Cause)`, `AccessDenied(Cause)`, `TlsFailure(Detail)` or `IoFailure(Cause)`.
The Cause variants retain [Std.Os.Errors.HostError](../host/errors.md) with the
qualified attempted operation, supplied resource identifiers, available native
code/domain and original explanation. DNS classification occurs at resolution;
I/O classification uses backend categories rather than diagnostic text. TLS
verification/protocol failures use TlsFailure; transport failures inside TLS
retain the corresponding native network cause.

Host deadline expiry also uses Timeout with a HostError, with absent native
code/domain when the backend supplied none. Input validation has no fabricated
host cause. Resources retain host then port for connect/listen, followed by PEM
paths for ListenTls; operations on a handle retain that handle's identifier.
Size, port and timeout validation use InvalidArgument. Byte-domain checks belong
to `Std.Bytes.FromArray`; socket input uses a trusted `Std.Bytes.ByteBuffer`.
EOF returns an empty buffer in `Ok`, partial writes return successful counts, and cancellation
before progress keeps its existing precedence and ownership. Match variants for
decisions; use Message() at deliberate text boundaries.

Import `Std.Bytes as ByteStorage` for buffer annotations or construction. Native
write jobs retain a stable copy-on-write value without copying individual bytes;
read jobs publish owned native storage directly.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Connection` | opaque VM-owned TCP or TLS connection |
| type | `Listener` | opaque VM-owned TCP or TLS listener |
| type | `NetworkAddress` | record with `Host: string` and `Port: integer` |
| function | `Connect(Host: string; Port: integer; TimeoutMillis: integer): Result of (Connection, NetError)` | resolves and connects |
| function | `ConnectTls(Host: string; Port: integer; TimeoutMillis: integer): Result of (Connection, NetError)` | resolves, connects, and completes a verified TLS handshake |
| function | `ConnectWithCancellation(Host: string; Port: integer; TimeoutMillis: integer; Token: Std.Tasks.CancellationToken): Result of (Connection, NetError)` | cancellation-aware DNS wait and TCP attempts |
| function | `ConnectTlsWithCancellation(Host: string; Port: integer; TimeoutMillis: integer; Token: Std.Tasks.CancellationToken): Result of (Connection, NetError)` | also observes cancellation during TLS handshake I/O |
| function | `Listen(Host: string; Port: integer): Result of (Listener, NetError)` | binds one TCP listener |
| function | `ListenTls(Host: string; Port: integer; CertificatePath: string; PrivateKeyPath: string; HandshakeTimeoutMillis: integer): Result of (Listener, NetError)` | loads PEM credentials and binds one TLS listener |
| function | `Accept(Listener): Result of (Connection, NetError)` | waits until one client connects |
| function | `AcceptWithCancellation(Listener; Token: Std.Tasks.CancellationToken): Result of (Connection, NetError)` | waits until one client connects or cancellation is requested |
| function | `CloseListener(Listener): Result of (boolean, NetError)` | invalidates the listener handle |
| function | `ListenerLocalAddress(Listener): Result of (NetworkAddress, NetError)` | returns the bound numeric IP address and port |
| function | `SetTimeout(Connection; TimeoutMillis: integer): Result of (boolean, NetError)` | sets read/write timeout; zero disables it |
| function | `ReceiveBytes(Connection; MaxBytes: integer): Result of (ByteStorage.ByteBuffer, NetError)` | empty buffer means EOF |
| function | `ReceiveBytesWithCancellation(Connection; MaxBytes: integer; Token: Std.Tasks.CancellationToken): Result of (ByteStorage.ByteBuffer, NetError)` | reads a chunk or reports cancellation; leaves the connection open |
| function | `SendBytes(Connection; Data: ByteStorage.ByteBuffer): Result of (integer, NetError)` | returns bytes written; partial writes are possible |
| function | `SendBytesWithCancellation(Connection; Data: ByteStorage.ByteBuffer; Token: Std.Tasks.CancellationToken): Result of (integer, NetError)` | returns accepted bytes or reports cancellation before progress |
| function | `Close(Connection): Result of (boolean, NetError)` | invalidates the handle |

`ConnectTls` verifies the server certificate and requested hostname through the operating system's
trust policy. It does not expose an insecure certificate bypass. The returned `Connection` uses the
same `ReceiveBytes`, `SendBytes`, `SetTimeout`, and `Close` functions as a plain TCP connection.

All connect variants use one monotonic timeout budget starting before native worker admission.
It includes the DNS wait, all TCP addresses, TLS configuration and the handshake. Call `SetTimeout`
to configure subsequent byte reads and writes. Synchronous OS TLS operations have the cancellation
limits described below.

`Listen` binds the requested local host and port. `ListenTls` first loads a PEM certificate chain and
PEM private key, then binds the requested address. Its positive handshake timeout is limited to five
minutes. Client certificates and multiple certificates selected through SNI are not supported.

`Accept` and `AcceptWithCancellation` return the same `Connection` type for TCP and TLS listeners,
so `SetTimeout`, `ReceiveBytes`, `SendBytes`, and `Close` apply to both. A TLS listener completes the handshake
before returning and discards failed or timed-out handshakes while waiting for a valid client.
Listener handles are closed separately with `CloseListener`.

## Task execution and resource ownership

In pool tasks and the default all-stop debugger, connects, TCP/TLS listener creation,
accepts, connection timeout configuration and byte reads/writes suspend the calling
FPAS task while an owned native worker performs the operation. Their
cancellation-aware forms use the same route.
Other FPAS tasks and debugger requests remain runnable, including with one VM
pool worker. Inline callbacks, such as array `Map` callbacks, suspend with their
owning task and retain their frames and mutable captures.

Each VM admits at most 64 pending native socket jobs. Completed results awaiting
their owner still occupy a slot. Saturation returns `NetError.IoFailure` with a
native cause describing the limit and no OS error code; admission does not wait
for a slot or create another thread. Retiring the job releases its slot.
Each byte transfer remains limited to 1 MiB. Pending operations use bounded native
offloading and the shared task timer's readiness probes.

Cancellation or abandonment joins the native worker before releasing its saved
task state. A new listener or accepted/outgoing connection not delivered to FPAS
is closed; abandoning a read/write or timeout configuration leaves its established
connection open. Already consumed or accepted
bytes are not rolled back when a task is abandoned. VM teardown cancels saved
waits and joins native work before returning.

The main program and synchronous native callback boundaries retain synchronous
host calls. Configuring a busy connection waits for its serialized I/O lock and
observes task abandonment while waiting. Cancellation does not undo a timeout
change that has already completed. Address queries and listener close retain their
direct execution path. Seeded scheduling rejects network effects rather than replaying them.

Connection `Close` invalidates the handle immediately. A busy transport is interrupted
out of band; an idle transport is detached from its aliases and queues TLS close_notify
when applicable. Pool/debugger tasks flush the detached transport through nonblocking
probes, preserving previously accepted bytes before closing the socket. This path needs
no native-job slot and remains available when all 64 socket jobs are occupied. The
configured write timeout bounds pending close probes; a successful flush takes
precedence over simultaneous expiry. Abandoning a pending close aborts its socket.

TLS listener configuration checks abandonment before loading PEM files, between
certificate entries, before loading the private key and before binding/publishing
the listener. An OS file read or cryptographic library call already in progress
cannot be forcibly interrupted; retiring the task joins that work. The handshake
timeout bounds accepted handshakes, not credential loading or listener DNS.

`AcceptWithCancellation` observes a `Std.Tasks.CancellationToken`. If cancellation wins before a
connection is returned, the function returns `Error(NetError.Cancelled(Operation))`. It leaves the
listener open, so another task may accept from it later. `Accept` remains available when the caller
does not need application-controlled cancellation.

Byte values must be in `0..255`. A single `ReceiveBytes` or `SendBytes` is limited to 1 MiB. Timeouts are limited
to `300000` milliseconds; `Connect` and `ConnectTls` require a positive timeout. Client and listener
ports differ: client ports must be in `1..65535`; listener ports accept `0..65535`.
Port `0` asks the operating system to assign an available port while binding the listener.

Connections and listeners belong to one VM, are shared safely
with its tasks, and are released when that VM ends. `Close` interrupts a blocked `ReceiveBytes` or `SendBytes`
on the connection, and `CloseListener` interrupts an active `Accept`, including a pending TLS
handshake. VM cancellation interrupts established connection I/O and listener waits before joining
task workers. All connect variants observe VM cancellation during resolution waits and pending
TCP/TLS I/O; synchronous TLS verification has the limits described below.
Closed handles cannot be reused.

FPAS code runs with the host process's network permissions. `Std.Net.Sockets` does not sandbox destinations.

### Name resolution

Numeric IPv4 and IPv6 hosts bypass DNS. Other hosts use the operating system resolver
on a native thread with at most 64 active lookups across the process. This separate
budget spans VM lifetimes; saturation returns `NetError.IoFailure` without an OS error code.

The DNS wait observes cancellation and, for connects, the shared connect deadline.
Listener creation has no DNS timeout parameter. If the wait ends,
the OS call may still be running: it retains only its native hostname, port and result
channel, with no VM state or FPAS values. Its slot remains occupied until the OS call
returns. Late addresses and errors are discarded and never connect or bind sockets.
Normally completed lookups are joined before returning their result.

## `NetworkAddress` and `ListenerLocalAddress`

`NetworkAddress` contains `Host: string` and `Port: integer`. For an open TCP or TLS
listener, `ListenerLocalAddress` returns its actual bound numeric IP address and port.
The port remains reserved by the listener; there is no separate probe-and-rebind step.
An address queried after `CloseListener` returns `Error`.

```pascal id=example-2 fragment context=declarations-net
function InspectListener(): result of (Net.NetworkAddress, Net.NetError);
begin
  const Server: Net.Listener := try Net.Listen('127.0.0.1', 0);
  const Address: Net.NetworkAddress := try Net.ListenerLocalAddress(Server);
  // Address.Host is '127.0.0.1'; Address.Port is the assigned nonzero port.
  const Closed: boolean := try Net.CloseListener(Server);
  return Ok(Address);
end function;
```

`ListenTls` supports the same port selection and address query after loading its TLS
credentials. `Host` contains an IPv6 address without brackets when bound to IPv6.
A wildcard binding returns `0.0.0.0` or `::`; this is the listening address, not an
advertised address for remote clients. Callers must choose a reachable host when
advertising a wildcard listener.

## `ConnectWithCancellation` and `ConnectTlsWithCancellation`

These variants take a `Std.Tasks.CancellationToken` and use the same host, port, and positive
`1..300000` ms timeout limits as `Connect` and `ConnectTls`. TLS uses the same operating-system trust
policy; cancellation does not bypass certificate or hostname verification.

- A token already cancelled prevents DNS resolution and socket creation for valid arguments.
- Cancellation is checked before and during the DNS wait, while each TCP attempt is pending,
  during TLS handshake I/O, and before publishing a handle. Observed cancellation returns
  `Error(NetError.Cancelled(Operation))`. Pending sockets are dropped without publishing a handle.
- All connect variants use one monotonic budget starting before worker admission, shared by DNS, all resolved
  addresses and the TLS handshake. Exhaustion observed at a checkpoint returns
  `Error(NetError.Timeout(Cause))`. A successful final checkpoint wins over cancellation arriving
  immediately afterward.
- Each address uses one non-blocking TCP attempt. Pending attempts check cancellation and the
  shared deadline between socket readiness checks, parking for at most 10 ms between checks.
  Cancellation or expiry closes that attempt's socket; polling does not restart the connection.
- **Cancellation is not a hard wall-clock bound:** the DNS wait can end while the isolated
  OS lookup continues under its process-wide limit. TLS platform configuration and certificate
  verification run synchronously inside the owned connect worker. Time spent there counts
  against the budget, but that worker can only observe cancellation or expiry after those calls
  return. TLS socket waits check cancellation between handshake I/O steps. Joining native work
  during abandonment or VM shutdown may therefore wait for synchronous TLS operations.
- Resolution, connection, and TLS failures remain errors rather than successful handles. If
  cancellation is observed at the same checkpoint as an OS error or expired budget, cancellation
  takes precedence. Successful connections initially have no read/write timeout; use `SetTimeout`.

## `ReceiveBytesWithCancellation`

Reads at most `MaxBytes` from an established TCP or TLS connection. The same byte-size limits and
EOF representation as `ReceiveBytes` apply. Import `Std.Tasks` to create the cancellation source and token.

- Cancellation observed before a read attempt returns `Error(NetError.Cancelled(Operation))` without
  closing the connection. A token that is already cancelled does not consume available bytes.
- A successful read wins over cancellation arriving during that read; its bytes are returned.
- The read timeout configured by `SetTimeout` bounds the read phase with one monotonic deadline;
  expiry returns `Error(NetError.Timeout(Cause))`. Zero disables that deadline, not cancellation.
- Operations on one connection are serialized. Cancellation is also checked while waiting for
  another operation to release the connection, but the read timeout starts after that wait.
- Cancellation and timeout preserve TLS state and the configured socket timeouts. Later reads may
  continue with a fresh token. Cancellation is polled at intervals of up to 10 ms between I/O
  attempts; OS scheduling can add latency. Pool tasks remain suspended during the
  native wait, as described above.

## `SendBytesWithCancellation`

Writes one bounded chunk to an established TCP or TLS connection. Byte values and the 1 MiB limit
are the same as for `SendBytes`. Import `Std.Tasks` to create the cancellation source and token.

- `Ok(N)` means the first `N` bytes were accepted by the local transport. Partial writes are allowed;
  retry only the remaining suffix. The function does not loop to send the entire input.
- Accepted bytes take priority over cancellation or timeout arriving during a write attempt. A
  positive count is never replaced by a cancellation error after those bytes have been accepted.
- Cancellation before an attempt returns `Error(NetError.Cancelled(Operation))`. No bytes from this
  call's input have been accepted. The connection stays open; previously accepted data is not undone.
- The write timeout from `SetTimeout` starts after acquiring the connection lock and uses one
  monotonic deadline across retries. Expiry before progress returns `Error(NetError.Timeout(Cause))`.
  Zero disables the deadline, not cancellation. Lock waits also observe cancellation.
- For TLS, accepted bytes can remain buffered locally. `Ok(N)` is not confirmation of delivery to
  the peer. A later operation may advance buffered output even if that later call is cancelled
  before accepting its own input. Never resend a prefix already reported as accepted.
- An empty input may return `Ok(0)`; a pre-cancelled token still returns cancellation. As with
  `SendBytes`, a zero count does not establish that the connection has closed.

Both cancellable I/O functions preserve socket timeout settings and normally restore blocking mode
before returning. If restoring that mode fails, the connection is invalidated; a successful byte
count or read result is still returned rather than losing progress information. Cancellation is
polled between I/O attempts with the same 10 ms interval as `ReceiveBytesWithCancellation`.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Cooperative task waits and bounded native jobs | [`wait.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/wait.rs) |
| VM connection registry | [`connections.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/connections.rs) |
| Cancellable connection establishment | [`establishment.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/connections/establishment.rs) |
| Bounded OS lookup lifetime | [`resolver.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/connections/establishment/resolver.rs) |
| Cancellable TCP attempts | [`establishment/tcp.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/connections/establishment/tcp.rs) |
| Cancellable reads | [`cancellable_read.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/connections/cancellable_read.rs) |
| Cancellable writes | [`cancellable_write.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/connections/cancellable_write.rs) |
| Cancellation and deadline polling | [`polling.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/connections/polling.rs) |
| VM listener registry | [`listeners.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/listeners.rs) |
| Cancellation registry | [`cancellation/registry.rs`](../../../../crates/fpas-runtime/src/execution/vm/cancellation/registry.rs) |
| TCP/TLS transport | [`transport.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/transport.rs) |
| TLS client verification | [`client.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/tls/client.rs) |
| TLS server credentials and handshake | [`server.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/tls/server.rs) |
| Hosted dispatch | [`net/mod.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/net/mod.rs) |
| Registration | [`loaded/net.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/net.rs) |
| Intrinsic ids | [`intrinsic/net.rs`](../../../../crates/fpas-code/src/intrinsic/net.rs) |

## See also

- [Networking index](README.md)
- [`Std.Net.Http`](http.md)
- [`Std.Tasks`](../concurrency/task.md)
- [Standard library index](../README.md)
