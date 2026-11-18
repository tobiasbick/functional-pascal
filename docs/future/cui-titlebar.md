# Optional CUI title bar

FPAS integration is open. The CUI fork branch `codex/cui-titlebar-testing`
(`762fbb9`) implements the title bar on Win32 and X11. The vendored CUI
revision lacks that API, and no upstream proposal has been made. See the
[desktop GUI application gaps](gui-app-gaps.md) for priorities.

## Goal and design

- Add an optional CUI title-bar component containing an icon, title, navigation,
  search and actions, followed by the window controls.
- Reuse existing CUI widgets. Keep native frame integration in the window
  backends, with drag regions that exclude interactive controls.
- On Windows, use existing Win32/DWM APIs and dynamically resolve optional OS
  functions. Avoid Windows App SDK, WinUI/XAML and Windows App Runtime
  dependencies. Microsoft's TitleBar control is a design reference, not a
  component to embed. Keep effects such as Mica outside the initial scope.
- On the branch, `cui_app_desc.custom_title_bar` drops the system title bar,
  and the chrome component `cui_title_bar(ui, id, .icon, .title) { ... }` draws
  the bar with CUI's own window buttons. Where the platform keeps its frame,
  the same call renders a bar without window buttons, so FPAS needs one layout.
  Platform layers report the frame through the optional
  `cui_platform_iface.window_frame` and `.window_button` hooks. They ask
  `cui_title_bar_hit_test` which points move the window.
- Keep the system frame as the default. Failed activation must leave it intact,
  and the title bar then renders as an ordinary bar.
- Preserve existing CUI APIs, public struct layouts and default behavior. Add
  separate functions/options if needed to retain binary compatibility.
- Ordinary browser/WASM hosts can draw an application header but cannot provide
  native browser-window title-bar integration.

## Delivery and validation

1. Verified on the branch: Windows 11 with both renderers covers window
   buttons, Snap layouts, dragging, double-click, edge resizing, dark mode,
   the inactive look and the system-frame fallback. Debian KDE KWin X11 with
   both renderers covers the same interactions, a 120-launch startup check
   and a one-pixel edge for the undecorated window. Untested: DPI changes
   while running, other X11 window managers and macOS.
2. Proposing the feature upstream requires explicit user authorization; none
   has been given. Expose the CUI feature through the FPAS binding only once
   the vendored CUI revision contains it.
3. Keep macOS on the system frame until a contributor can implement and validate
   it on macOS. CI compilation alone does not verify native window interaction.
   Validate frame behavior in native desktop sessions; WASM page screenshots do
   not validate native window decorations.

## References

- [Windows title-bar customization](https://learn.microsoft.com/en-us/windows/apps/develop/title-bar)
- [Microsoft TitleBar control](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/title-bar)
- [Win32 custom window frames](https://learn.microsoft.com/en-us/windows/win32/dwm/customframe)
- [CUI contribution guidelines](https://github.com/yappologistic/CUI/blob/master/docs/CONTRIBUTING.md)
