//! Shared `Std.Graph` lifecycle and hosted dispatch state.

use fpas_bytecode::Value;
use fpas_std::{GraphHost, GraphSession, UiHost};

/// Shared `Std.Graph` lifecycle and hosted-dispatch state for the active VM.
#[derive(Debug)]
pub(crate) struct GraphState {
    /// Current graph session and runtime-owned backbuffer state.
    pub session: GraphSession,
    /// Resize coalescing and the hosted-loop event pump (`docs/pascal/std/graph/app/README.md`).
    pub host: GraphHost,
    /// `OnKeyPressed`-style handler: `function (Application, KeyEvent): boolean`.
    pub on_key_pressed: Option<Value>,
    /// `OnMouse`-style handler: `procedure (Application, Event)`.
    pub on_mouse: Option<Value>,
    /// `OnWheel`-style handler: `procedure (Application, Event)`.
    pub on_wheel: Option<Value>,
    /// `OnResize`-style handler: `procedure (Application, Size)`.
    pub on_resize: Option<Value>,
    /// `OnCloseRequested`-style handler: `procedure (Application)`.
    pub on_close_requested: Option<Value>,
    /// `OnPaint`-style handler: `procedure (Application)`.
    pub on_paint: Option<Value>,
    /// `OnIdle`-style handler: `procedure (Application)`.
    pub on_idle: Option<Value>,
    /// Idle interval for hosted `Application.Run` in milliseconds; `0` disables idle.
    pub idle_interval_ms: i64,
    /// `OnExit`-style handler: `procedure (Application, ExitReason)`.
    pub on_exit: Option<Value>,
    /// Last reason recorded for a hosted run.
    pub last_exit_reason: Option<Value>,
    /// Set by `Application.HostRequestQuit`.
    pub quit_requested: bool,
    /// Set when the native window requests close during a hosted run.
    pub window_closed: bool,
    /// Set when low-level code asks the active hosted run to stop.
    pub host_stop_requested: bool,
    /// Guards the single hosted `Application.Run` entrypoint.
    pub run_active: bool,
    /// Test-only events queued before `Application.Open`.
    pub pending_test_events: Vec<fpas_std::GraphEvent>,
    /// Whether the active session was opened with `Application.OpenForTest`.
    pub headless_test_open: bool,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            session: GraphSession::default(),
            host: UiHost::for_graph(),
            on_key_pressed: None,
            on_mouse: None,
            on_wheel: None,
            on_resize: None,
            on_close_requested: None,
            on_paint: None,
            on_idle: None,
            idle_interval_ms: 0,
            on_exit: None,
            last_exit_reason: None,
            quit_requested: false,
            window_closed: false,
            host_stop_requested: false,
            run_active: false,
            pending_test_events: Vec::new(),
            headless_test_open: false,
        }
    }
}
