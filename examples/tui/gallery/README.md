# TUI Widget Gallery

Choose a widget from the list on the left and try it in the preview on the
right. Use the theme buttons below the preview to switch between Light, Dark,
Green and Blue. The selected page and its values stay in place.

After the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application),
build and run the gallery from the repository root in a terminal.

```sh
dotnet build examples/tui/gallery/WidgetGallery.fpasproj
dotnet examples/tui/gallery/bin/Debug/net10.0/WidgetGallery.dll
```

Use the mouse or select a page with the arrow keys and Enter. Page Up and
Page Down switch pages when focus is outside the list and scroll view previews. Tab moves focus between
controls and Ctrl+Q exits. The overlay captures focus until you close it with
Escape or its Close button.

| Page | Try it |
| --- | --- |
| Buttons | Press the enabled button and compare it with the disabled one. |
| Checkbox | Toggle an option and inspect a disabled checked option. |
| Text input | Type, select, or paste several lines into a single-line field. |
| Lists | Select a row with the mouse or arrows and open it with Enter. Cedar cannot be opened. Scroll through the rows or toggle the empty state. |
| Progress bar | Advance from 0 to 100 percent and reset. |
| Text & layout | Resize the terminal and compare fixed and growing children. |
| Scroll view | Append events to the log. Scroll up to pause following and press End in the viewport to resume. |
| Cell canvas | Resize the viewport and try clicks, middle clicks and the wheel. |
| Overlay | Open the overlay, resize the window, then close it and check focus. |

The gallery uses the same model and views in the
[private debug terminal](../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
and in a headless session. For manual headless stepping, launch it with
`--headless-control` and set `FPAS_TUI_CONTROL_PIPE` to your controller's
current-user pipe named `fpas-tui-<32 hex digits>`. The session starts at 80 × 24
cells. Use the pipe protocol documented in the
[filter list example](../filter-list/README.md).

For example, activate `pages/lists` and then `sample-list` to open Birch.
Send Down and step after each action. The snapshot reports `cedar` as the value of
`sample-list`. Activate `theme-blue` and step again to see the same selection
in the Blue theme. Add new previews in `Previews.fpas` and register their pages
in `State.fpas`.
