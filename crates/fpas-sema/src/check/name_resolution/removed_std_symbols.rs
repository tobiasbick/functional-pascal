//! Hints for standard-library symbols intentionally removed from the public surface.

const TUI_REMOVED_HINT: &str = "The old try-1 Std.Tui host/view API was removed during the Turbo Vision rewrite. Use the try-2 facade: Application.Open or OpenForTest, Dialog.NewModal, Button.New, Dialog.Add, Window.New, Desktop.Add, Application.Run(App, OnCommand), Application.ExecView, and CM_* command constants.";

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
    "Application.Create",
    "Application.AddChild",
    "Application.AddWindow",
    "Application.ExecDialog",
    "Application.InputText",
    "Application.Checked",
    "Application.Selected",
    "Application.ListSelection",
    "Application.OutlineSelection",
    "Application.OutlineSelectedText",
    "Application.SetText",
    "Application.SetChecked",
    "Application.SetItems",
    "Application.SetOutlineNodes",
    "Application.SetTitle",
    "Application.SetMenus",
    "Application.SetStatusItems",
    "Application.OnCommand",
    "Application.Pump",
    "Command.",
    "Dialog.AddButton",
    "Application.Try2Inject",
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
