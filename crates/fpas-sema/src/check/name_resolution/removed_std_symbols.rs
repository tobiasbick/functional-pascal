//! Hints for standard-library symbols intentionally removed from the public surface.

const TUI_REMOVED_HINT: &str = "The old retained Std.Tui host/view API was removed during the Turbo Vision rewrite. Use the current facade: Application.Open or OpenForTest, Application.CreateDialog, Application.CreateButton, Application.AddChild, Application.OnCommand, Application.Pump, Application.Run, and Application.Quit.";

const REMOVED_TUI_PREFIXES: &[&str] = &[
    "Application.Host",
    "Application.QueryView",
    "Application.QueryResolvedView",
    "Application.QuerySceneGraph",
    "Application.QueryMenuBarState",
    "Application.QueryModalDepth",
    "Application.QueryFocusedViewId",
    "Application.QueryFrame",
    "Application.ShowFramedDialog",
    "Application.ShowModal",
    "Application.ShowDialog",
    "Application.CloseModal",
];

/// Returns a migration hint when `name` targets a removed `Std.Tui` callable.
pub(crate) fn hint_for_removed_std_callable(name: &str) -> Option<&'static str> {
    let canonical_name = name.to_ascii_lowercase();
    let stripped = canonical_name
        .strip_prefix("std.tui.")
        .unwrap_or(canonical_name.as_str());
    REMOVED_TUI_PREFIXES
        .iter()
        .any(|prefix| stripped.starts_with(&prefix.to_ascii_lowercase()))
        .then_some(TUI_REMOVED_HINT)
}
