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
| function | `Listen(Host: string; Port: integer): Result of Listener, string` | binds one TCP listener |
| function | `ListenTls(Host: string; Port: integer; CertificatePath: string; PrivateKeyPath: string; HandshakeTimeoutMillis: integer): Result of Listener, string` | loads PEM credentials and binds one TLS listener |
| function | `Accept(Listener): Result of Connection, string` | blocks until one client connects |
| function | `CloseListener(Listener): Result of boolean, string` | invalidates the listener handle |
| function | `SetTimeout(Connection; TimeoutMillis: integer): Result of boolean, string` | sets read/write timeout; zero disables it |
| function | `Read(Connection; MaxBytes: integer): Result of array of integer, string` | empty array means EOF |
| function | `Write(Connection; Data: array of integer): Result of integer, string` | returns bytes written; partial writes are possible |
| function | `Close(Connection): Result of boolean, string` | invalidates the handle |

`ConnectTls` verifies the server certificate and requested hostname through the operating system's
trust policy. It does not expose an insecure certificate bypass. The returned `Connection` uses the
same `Read`, `Write`, `SetTimeout`, and `Close` functions as a plain TCP connection.

The connect timeout bounds TCP establishment and, for `ConnectTls`, the TLS handshake. Call
`SetTimeout` to configure subsequent byte reads and writes.

`Listen` binds the requested local host and port. `ListenTls` first loads a PEM certificate chain and
PEM private key, then binds the requested address. Its positive handshake timeout is limited to five
minutes. Client certificates and multiple certificates selected through SNI are not supported.

`Accept` returns the same `Connection` type for TCP and TLS listeners, so `SetTimeout`, `Read`,
`Write`, and `Close` apply to both. A TLS listener completes the handshake before returning and
discards failed or timed-out handshakes while waiting for a valid client. Listener handles are
closed separately with `CloseListener`. Both `Accept` and connection I/O block the VM worker that
executes them.

Byte values must be in `0..255`. A single `Read` or `Write` is limited to 1 MiB. Timeouts are limited
to `300000` milliseconds; `Connect` and `ConnectTls` require a positive timeout. Client and listener
ports must be in `1..65535`.

Calls block their VM worker thread. Connections and listeners belong to one VM, are shared safely
with its tasks, and are released when that VM ends. `Close` interrupts a blocked `Read` or `Write`
on the connection, and `CloseListener` interrupts an active `Accept`, including a pending TLS
handshake. VM cancellation interrupts all blocking network operations before joining task workers.
Closed handles cannot be reused.

FPAS code runs with the host process's network permissions. `Std.Net` does not sandbox destinations.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| VM connection registry | [`connections.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections.rs) |
| VM listener registry | [`listeners.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/listeners.rs) |
| TCP/TLS transport | [`transport.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/transport.rs) |
| TLS client verification | [`client.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/tls/client.rs) |
| TLS server credentials and handshake | [`server.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/tls/server.rs) |
| Hosted dispatch | [`net/mod.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/mod.rs) |
| Registration | [`loaded/net.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/net.rs) |
| Intrinsic ids | [`intrinsic/net.rs`](../../../../crates/fpas-bytecode/src/intrinsic/net.rs) |

## See also

- [Networking index](README.md)
- [`Std.Http`](http.md)
