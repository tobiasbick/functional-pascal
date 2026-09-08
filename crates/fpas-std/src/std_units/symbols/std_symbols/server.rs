//! Server lifetime symbol names. See `docs/pascal/std/network/server.md`.

std_symbol!(STD_SERVER_LIFETIME = "Std.Server.ServerLifetime");
std_symbol!(STD_SERVER_CREATE_LIFETIME = "Std.Server.CreateLifetime");
std_symbol!(STD_SERVER_GET_WORK_GROUP = "Std.Server.GetWorkGroup");
std_symbol!(STD_SERVER_GET_STOP_TOKEN = "Std.Server.GetStopToken");
std_symbol!(STD_SERVER_IS_READY = "Std.Server.IsReady");
std_symbol!(STD_SERVER_REQUEST_STOP = "Std.Server.RequestStop");
std_symbol!(STD_SERVER_REMAINING_MILLIS = "Std.Server.RemainingMillis");
std_symbol!(STD_SERVER_OWN_LISTENER = "Std.Server.OwnListener");
std_symbol!(STD_SERVER_FINISH_SHUTDOWN = "Std.Server.FinishShutdown");
std_symbol!(STD_SERVER_OBSERVE_SIGNALS = "Std.Server.ObserveSignals");
std_symbol!(STD_SERVER_SHUTDOWN_ERRORS = "Std.Server.ShutdownErrors");

pub(in crate::std_units) const STD_SERVER_SYMBOLS: &[&str] = &[
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
