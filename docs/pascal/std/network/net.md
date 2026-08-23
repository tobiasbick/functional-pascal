# `Std.Net`

Hosted blocking TCP connections with explicit timeouts and byte arrays.

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
| type | `Connection` | opaque VM-owned TCP connection |
| function | `Connect(Host: string; Port: integer; TimeoutMillis: integer): Result of Connection, string` | resolves and connects |
| function | `SetTimeout(Connection; TimeoutMillis: integer): Result of boolean, string` | sets read/write timeout; zero disables it |
| function | `Read(Connection; MaxBytes: integer): Result of array of integer, string` | empty array means EOF |
| function | `Write(Connection; Data: array of integer): Result of integer, string` | returns bytes written; partial writes are possible |
| function | `Close(Connection): Result of boolean, string` | invalidates the handle |

Byte values must be in `0..255`. A single `Read` or `Write` is limited to 1 MiB. Timeouts are limited to `300000` milliseconds; `Connect` requires a positive timeout. Ports must be in `1..65535`.

Calls block their VM worker thread. Connections belong to one VM, are shared safely with its tasks, and are released when that VM ends. A closed connection cannot be reused.

FPAS code runs with the host process's network permissions. `Std.Net` does not sandbox destinations.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| VM connection registry | [`connections.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/connections.rs) |
| Hosted dispatch | [`net/mod.rs`](../../../../crates/fpas-vm/src/vm/hosted/net/mod.rs) |
| Registration | [`loaded/net.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/net.rs) |
| Intrinsic ids | [`intrinsic/net.rs`](../../../../crates/fpas-bytecode/src/intrinsic/net.rs) |

## See also

- [Networking index](README.md)
- [`Std.Http`](http.md)
