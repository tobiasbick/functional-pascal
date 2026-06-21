//! `Std.Tui` retained view-state mutation symbols.
//!
//! **Documentation:** `docs/pascal/std/tui/app/views.md`

/// Set whether a retained view and its descendants are visible.
pub const STD_TUI_APPLICATION_HOST_SET_VIEW_VISIBLE: &str =
    std_tui!("Application.HostSetViewVisible");

/// Set whether a retained view accepts input and focus.
pub const STD_TUI_APPLICATION_HOST_SET_VIEW_ENABLED: &str =
    std_tui!("Application.HostSetViewEnabled");
