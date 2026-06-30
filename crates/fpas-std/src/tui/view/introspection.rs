//! Stable view-kind metadata exposed through scene-graph introspection.
//!
//! **Documentation:** `docs/pascal/std/tui/app/views.md`

/// Pascal-visible `Std.Tui.ViewKind` variants in runtime discriminant order.
pub const TUI_VIEW_KIND_VARIANTS: &[&str] =
    &["Generic", "SolidFill", "MenuBar", "StatusBar", "Frame"];

/// Kind of retained view content attached to a scene-graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum ViewKind {
    /// View without a native widget.
    Generic,
    /// Solid-fill widget.
    SolidFill,
    /// Menu-bar widget.
    MenuBar,
    /// Status-bar widget.
    StatusBar,
    /// Host-painted window or dialog frame.
    Frame,
}
