//! `Std.Tui` frame-root symbols.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

pub const STD_TUI_FRAME_ROOT_STATE: &str = std_tui!("FrameRootState");
pub const STD_TUI_APPLICATION_HOST_SET_DESKTOP_WORK_AREA: &str =
    std_tui!("Application.HostSetDesktopWorkArea");
/// Create a retained frame with host-painted chrome.
pub const STD_TUI_APPLICATION_HOST_CREATE_FRAME_VIEW: &str =
    std_tui!("Application.HostCreateFrameView");
/// Create an owned painted frame and enter it as the active modal dialog.
pub const STD_TUI_APPLICATION_SHOW_FRAMED_DIALOG: &str = std_tui!("Application.ShowFramedDialog");
pub const STD_TUI_APPLICATION_HOST_ACTIVATE_NEXT_WINDOW: &str =
    std_tui!("Application.HostActivateNextWindow");
pub const STD_TUI_APPLICATION_HOST_ZOOM_FRAME_ROOT: &str =
    std_tui!("Application.HostZoomFrameRoot");
pub const STD_TUI_APPLICATION_HOST_RESTORE_FRAME_ROOT: &str =
    std_tui!("Application.HostRestoreFrameRoot");
pub const STD_TUI_APPLICATION_QUERY_FRAME_ROOT_STATE: &str =
    std_tui!("Application.QueryFrameRootState");
pub const STD_TUI_APPLICATION_HOST_CASCADE_FRAME_ROOTS: &str =
    std_tui!("Application.HostCascadeFrameRoots");
pub const STD_TUI_APPLICATION_HOST_TILE_FRAME_ROOTS: &str =
    std_tui!("Application.HostTileFrameRoots");
pub const STD_TUI_FRAME_SCROLL_STATE: &str = std_tui!("FrameScrollState");
pub const STD_TUI_FRAME_WINDOW_ENTRY: &str = std_tui!("FrameWindowEntry");
pub const STD_TUI_APPLICATION_QUERY_FRAME_WINDOW_LIST: &str =
    std_tui!("Application.QueryFrameWindowList");
pub const STD_TUI_APPLICATION_HOST_ACTIVATE_FRAME_WINDOW: &str =
    std_tui!("Application.HostActivateFrameWindow");
pub const STD_TUI_APPLICATION_HOST_SET_FRAME_CONTENT_SIZE: &str =
    std_tui!("Application.HostSetFrameContentSize");
pub const STD_TUI_APPLICATION_HOST_SCROLL_FRAME: &str = std_tui!("Application.HostScrollFrame");
pub const STD_TUI_APPLICATION_HOST_SET_FRAME_SCROLL_OFFSET: &str =
    std_tui!("Application.HostSetFrameScrollOffset");
pub const STD_TUI_APPLICATION_QUERY_FRAME_SCROLL_STATE: &str =
    std_tui!("Application.QueryFrameScrollState");
