//! `Std.Graph` symbol names and registry group.

pub const STD_GRAPH_APPLICATION: &str = std_graph!("Application");
/// Hosted-dispatch handler bundle for `Std.Graph.Application.Configure`; see `docs/pascal/std/graph/app/README.md`.
pub const STD_GRAPH_APPLICATION_HANDLERS: &str = std_graph!("ApplicationHandlers");
pub const STD_GRAPH_SIZE: &str = std_graph!("Size");
pub const STD_GRAPH_EVENT: &str = std_graph!("Event");
pub const STD_GRAPH_EVENT_KIND: &str = std_graph!("EventKind");
pub const STD_GRAPH_EXIT_REASON: &str = std_graph!("ExitReason");
pub const STD_GRAPH_APPLICATION_OPEN: &str = std_graph!("Application.Open");
pub const STD_GRAPH_APPLICATION_CLOSE: &str = std_graph!("Application.Close");
pub const STD_GRAPH_APPLICATION_CONFIGURE: &str = std_graph!("Application.Configure");
pub const STD_GRAPH_APPLICATION_RUN: &str = std_graph!("Application.Run");
pub const STD_GRAPH_APPLICATION_SIZE: &str = std_graph!("Application.Size");
pub const STD_GRAPH_APPLICATION_REQUEST_REDRAW: &str = std_graph!("Application.RequestRedraw");
pub const STD_GRAPH_APPLICATION_HOST_REQUEST_QUIT: &str = std_graph!("Application.HostRequestQuit");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED: &str =
    std_graph!("Application.HostRegisterOnKeyPressed");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_RESIZE: &str =
    std_graph!("Application.HostRegisterOnResize");
pub const STD_GRAPH_APPLICATION_HOST_PROCESS_NEXT: &str = std_graph!("Application.HostProcessNext");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_PAINT: &str =
    std_graph!("Application.HostRegisterOnPaint");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_IDLE: &str =
    std_graph!("Application.HostRegisterOnIdle");
pub const STD_GRAPH_APPLICATION_HOST_DISPATCH_REDRAW: &str =
    std_graph!("Application.HostDispatchRedraw");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_EXIT: &str =
    std_graph!("Application.HostRegisterOnExit");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_MOUSE: &str =
    std_graph!("Application.HostRegisterOnMouse");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_WHEEL: &str =
    std_graph!("Application.HostRegisterOnWheel");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_CLOSE_REQUESTED: &str =
    std_graph!("Application.HostRegisterOnCloseRequested");
pub const STD_GRAPH_APPLICATION_UPLOAD_FRAME: &str = std_graph!("Application.UploadFrame");
pub const STD_GRAPH_APPLICATION_CLEAR: &str = std_graph!("Application.Clear");
pub const STD_GRAPH_APPLICATION_PUT_PIXEL: &str = std_graph!("Application.PutPixel");
pub const STD_GRAPH_APPLICATION_PRESENT: &str = std_graph!("Application.Present");
pub const STD_GRAPH_APPLICATION_DRAW_LINE: &str = std_graph!("Application.DrawLine");
pub const STD_GRAPH_APPLICATION_DRAW_RECT: &str = std_graph!("Application.DrawRect");
pub const STD_GRAPH_APPLICATION_FILL_RECT: &str = std_graph!("Application.FillRect");
pub const STD_GRAPH_APPLICATION_DRAW_CIRCLE: &str = std_graph!("Application.DrawCircle");
pub const STD_GRAPH_APPLICATION_DRAW_TEXT: &str = std_graph!("Application.DrawText");
pub const STD_GRAPH_APPLICATION_OPEN_FOR_TEST: &str = std_graph!("Application.OpenForTest");
pub const STD_GRAPH_APPLICATION_TEST_SEND_KEY: &str = std_graph!("Application.TestSendKey");

pub(in crate::std_units) const STD_GRAPH_SYMBOLS: &[&str] = &[
    STD_GRAPH_APPLICATION,
    STD_GRAPH_APPLICATION_HANDLERS,
    STD_GRAPH_SIZE,
    STD_GRAPH_EVENT,
    STD_GRAPH_EVENT_KIND,
    STD_GRAPH_EXIT_REASON,
    STD_GRAPH_APPLICATION_OPEN,
    STD_GRAPH_APPLICATION_CLOSE,
    STD_GRAPH_APPLICATION_CONFIGURE,
    STD_GRAPH_APPLICATION_RUN,
    STD_GRAPH_APPLICATION_SIZE,
    STD_GRAPH_APPLICATION_REQUEST_REDRAW,
    STD_GRAPH_APPLICATION_HOST_REQUEST_QUIT,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_RESIZE,
    STD_GRAPH_APPLICATION_HOST_PROCESS_NEXT,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_PAINT,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_IDLE,
    STD_GRAPH_APPLICATION_HOST_DISPATCH_REDRAW,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_EXIT,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_MOUSE,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_WHEEL,
    STD_GRAPH_APPLICATION_HOST_REGISTER_ON_CLOSE_REQUESTED,
    STD_GRAPH_APPLICATION_UPLOAD_FRAME,
    STD_GRAPH_APPLICATION_CLEAR,
    STD_GRAPH_APPLICATION_PUT_PIXEL,
    STD_GRAPH_APPLICATION_PRESENT,
    STD_GRAPH_APPLICATION_DRAW_LINE,
    STD_GRAPH_APPLICATION_DRAW_RECT,
    STD_GRAPH_APPLICATION_FILL_RECT,
    STD_GRAPH_APPLICATION_DRAW_CIRCLE,
    STD_GRAPH_APPLICATION_DRAW_TEXT,
    STD_GRAPH_APPLICATION_OPEN_FOR_TEST,
    STD_GRAPH_APPLICATION_TEST_SEND_KEY,
];
