//! `Std.Tui` frame-root symbols.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

pub const STD_TUI_FRAME_ROOT_STATE: &str = std_tui!("FrameRootState");
pub const STD_TUI_APPLICATION_HOST_SET_DESKTOP_WORK_AREA: &str =
    std_tui!("Application.HostSetDesktopWorkArea");
/// Create a retained frame with host-painted chrome.
pub const STD_TUI_APPLICATION_HOST_CREATE_FRAME_VIEW: &str =
    std_tui!("Application.HostCreateFrameView");
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
