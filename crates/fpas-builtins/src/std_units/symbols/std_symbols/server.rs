//! Server lifetime symbol names. See `docs/pascal/std/network/server.md`.

/// Native operational error family.
pub const STD_SERVER_ERROR: &str = "Std.Net.Server.ServerError";
std_symbol!(STD_SERVER_LIFETIME = "Std.Net.Server.ServerLifetime");
std_symbol!(STD_SERVER_CREATE_LIFETIME = "Std.Net.Server.CreateLifetime");
std_symbol!(STD_SERVER_GET_WORK_GROUP = "Std.Net.Server.GetWorkGroup");
std_symbol!(STD_SERVER_GET_STOP_TOKEN = "Std.Net.Server.GetStopToken");
std_symbol!(STD_SERVER_IS_READY = "Std.Net.Server.IsReady");
std_symbol!(STD_SERVER_REQUEST_STOP = "Std.Net.Server.RequestStop");
std_symbol!(STD_SERVER_REMAINING_MILLIS = "Std.Net.Server.RemainingMillis");
std_symbol!(STD_SERVER_OWN_LISTENER = "Std.Net.Server.OwnListener");
std_symbol!(STD_SERVER_FINISH_SHUTDOWN = "Std.Net.Server.FinishShutdown");
std_symbol!(STD_SERVER_OBSERVE_SIGNALS = "Std.Net.Server.ObserveSignals");
std_symbol!(STD_SERVER_SHUTDOWN_ERRORS = "Std.Net.Server.ShutdownErrors");

pub(in crate::std_units) const STD_SERVER_SYMBOLS: &[&str] = &[
    STD_SERVER_ERROR,
    STD_SERVER_LIFETIME,
    STD_SERVER_CREATE_LIFETIME,
    STD_SERVER_GET_WORK_GROUP,
    STD_SERVER_GET_STOP_TOKEN,
    STD_SERVER_IS_READY,
    STD_SERVER_REQUEST_STOP,
    STD_SERVER_REMAINING_MILLIS,
    STD_SERVER_OWN_LISTENER,
    STD_SERVER_FINISH_SHUTDOWN,
    STD_SERVER_OBSERVE_SIGNALS,
    STD_SERVER_SHUTDOWN_ERRORS,
];
