//! `Std.Net.Sockets` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/network/net.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Net.Sockets.*`.
///
/// **Documentation:** `docs/pascal/std/network/net.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum NetIntrinsic {
    /// `Std.Net.Sockets.Connect(Host, Port, TimeoutMillis)` - open one TCP connection.
    Connect = 518,
    /// `Std.Net.Sockets.SetTimeout(Connection, TimeoutMillis)` - set read and write timeouts.
    SetTimeout = 519,
    /// `Std.Net.Sockets.ReceiveBytes(Connection, MaxBytes)` - receive one byte chunk.
    Read = 520,
    /// `Std.Net.Sockets.SendBytes(Connection, Data)` - send one byte chunk.
    Write = 521,
    /// `Std.Net.Sockets.Close(Connection)` - close and invalidate one connection.
    Close = 522,
    /// `Std.Net.Sockets.ConnectTls(Host, Port, TimeoutMillis)` - open one verified TLS connection.
    ConnectTls = 523,
    /// `Std.Net.Sockets.Listen(Host, Port)` - bind one TCP listener.
    Listen = 524,
    /// `Std.Net.Sockets.Accept(Listener)` - accept one TCP connection.
    Accept = 525,
    /// `Std.Net.Sockets.CloseListener(Listener)` - close and invalidate one listener.
    CloseListener = 526,
    /// `Std.Net.Sockets.ListenTls(Host, Port, CertificatePath, PrivateKeyPath,
    /// HandshakeTimeoutMillis)` - bind one TLS listener.
    ListenTls = 527,
    /// `Std.Net.Sockets.AcceptWithCancellation(Listener, Token)` - accept until cancellation.
    AcceptWithCancellation = 536,
    /// `Std.Net.Sockets.ReceiveBytesWithCancellation(Connection, MaxBytes, Token)` - read until cancellation.
    ReadWithCancellation = 547,
    /// `Std.Net.Sockets.SendBytesWithCancellation(Connection, Data, Token)` - write until cancellation.
    WriteWithCancellation = 548,
    /// `Std.Net.Sockets.ConnectWithCancellation(Host, Port, TimeoutMillis, Token)`.
    ConnectWithCancellation = 549,
    /// `Std.Net.Sockets.ConnectTlsWithCancellation(Host, Port, TimeoutMillis, Token)`.
    ConnectTlsWithCancellation = 550,
    /// `Std.Net.Sockets.ListenerLocalAddress(Listener)` - read the bound host and port.
    ListenerLocalAddress = 566,
}
}
