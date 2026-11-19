# App chrome

The gallery places menus, tools, tabs and a status bar around an editor.
A second example shows dockable panes and a third shows a two-pane split.
A fourth changes and reads the window's state and placement.
These widgets and commands require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Menu bar](menu-bar.md) | Group application commands in menus. |
| [Document tabs](document-tabs.md) | Select, close and reorder open documents. |
| [Status bar](status-bar.md) | Present progress, state and clickable status items. |
| [Docking workspace](dockspace.md) | Arrange tabbed panels and save the layout. |
| [Split view](split-view.md) | Resize two panes with a divider. |
| [Window state and placement](window.md) | Maximize, minimize, restore or move the window and reopen it where it was. |
| [Toolbar](../canvas-and-bars/toolbar.md) | Arrange editing commands, toggle state and primary actions. |

## See also

- [UI handbook](../README.md)
- [App chrome in the gallery](../../../../examples/ui/gui/gallery/Pages/Desktop/View.fpas)

## Source files

- `examples/ui/gui/gallery/Pages/Desktop/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Desktop/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Desktop/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/Hosting/`
  - [CuiWindow.ChromeScopes.cs](../../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.ChromeScopes.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [desktop_chrome.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/desktop_chrome.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/desktop/`
  - [chrome_input.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/desktop/chrome_input.h)
- `tests/Fixtures/gui-host/`
  - [DesktopChromeContract.fpas](../../../../tests/Fixtures/gui-host/DesktopChromeContract.fpas)
