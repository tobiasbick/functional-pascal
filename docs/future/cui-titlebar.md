# Optional CUI title bar

Proposal only; native title-bar behavior has not been implemented or tested.

## Goal and design

- Add an optional CUI title-bar component containing an icon, title, navigation,
  search and actions, with space reserved for system window controls.
- Reuse existing CUI widgets. Keep native frame integration in the window
  backends, with drag regions that exclude interactive controls.
- On Windows, use existing Win32/DWM APIs and dynamically resolve optional OS
  functions. Avoid Windows App SDK, WinUI/XAML and Windows App Runtime
  dependencies. Microsoft's TitleBar control is a design reference, not a
  component to embed. Keep effects such as Mica outside the initial scope.
- Candidate API: `cui_app_try_enable_titlebar(app)` returns whether native
  integration was activated for that window. Names and activation timing remain
  open. A DLL-presence check alone is insufficient; check the backend's actual
  capabilities and initialization results. Perform setup once, outside rendering.
- Keep the system frame as the default. Failed activation must leave it intact;
  the application chooses a fallback layout, such as an ordinary toolbar.
- Preserve existing CUI APIs, public struct layouts and default behavior. Add
  separate functions/options if needed to retain binary compatibility.
- Ordinary browser/WASM hosts can draw an application header but cannot provide
  native browser-window title-bar integration.

## Delivery and validation

1. Propose the feature upstream before committing to a public API. Discuss
   scope, frame metrics, drag regions and compatibility; offer an implementation.
   Publishing an issue or PR requires explicit user authorization.
2. Work in focused PRs, starting with Windows and the Linux backends we can test.
   Expose the accepted CUI feature through the FPAS binding afterward; keep the
   vendored CUI release unchanged until a deliberate dependency update.
3. Test Windows on the native host: system buttons, Snap, dragging, resizing,
   maximize/restore, DPI changes, focus and interactive header content. Check
   both CUI renderers and the unsupported/failed-activation fallback.
4. Test Linux in Debian Sid under VirtualBox, recording whether the session uses
   X11 or Wayland. WSL with WSLg, when available, provides supplementary coverage.
   Assess SDL2/GLFW support separately; CUI has no native Wayland backend.
5. Keep macOS on the system frame until a contributor can implement and validate
   it on macOS. CI compilation alone does not verify native window interaction.
   Validate frame behavior in native desktop sessions; WASM page screenshots do
   not validate native window decorations.

## References

- [Windows title-bar customization](https://learn.microsoft.com/en-us/windows/apps/develop/title-bar)
- [Microsoft TitleBar control](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/title-bar)
- [Win32 custom window frames](https://learn.microsoft.com/en-us/windows/win32/dwm/customframe)
- [CUI contribution guidelines](https://github.com/yappologistic/CUI/blob/master/docs/CONTRIBUTING.md)
