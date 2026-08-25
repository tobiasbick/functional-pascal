//! `Std.Net` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/network/net.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Net.*`.
///
/// **Documentation:** `docs/pascal/std/network/net.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum NetIntrinsic {
    /// `Std.Net.Connect(Host, Port, TimeoutMillis)` - open one TCP connection.
    Connect = 518,
    /// `Std.Net.SetTimeout(Connection, TimeoutMillis)` - set read and write timeouts.
    SetTimeout = 519,
    /// `Std.Net.Read(Connection, MaxBytes)` - read one byte chunk.
    Read = 520,
    /// `Std.Net.Write(Connection, Data)` - write one byte chunk.
    Write = 521,
    /// `Std.Net.Close(Connection)` - close and invalidate one connection.
    Close = 522,
    /// `Std.Net.ConnectTls(Host, Port, TimeoutMillis)` - open one verified TLS connection.
    ConnectTls = 523,
    /// `Std.Net.Listen(Host, Port)` - bind one TCP listener.
    Listen = 524,
    /// `Std.Net.Accept(Listener)` - accept one TCP connection.
    Accept = 525,
    /// `Std.Net.CloseListener(Listener)` - close and invalidate one listener.
    CloseListener = 526,
}
}
