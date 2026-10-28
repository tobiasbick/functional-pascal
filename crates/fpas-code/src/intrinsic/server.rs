//! Server lifetime operations. See `docs/pascal/std/network/server.md`.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Hosted server lifetime operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ServerIntrinsic {
    CreateLifetime = 600,
    GetWorkGroup = 601,
    GetStopToken = 602,
    IsReady = 603,
    RequestStop = 604,
    RemainingMillis = 605,
    OwnListener = 606,
    FinishShutdown = 607,
    ObserveSignals = 608,
    ShutdownErrors = 609,
}
}
