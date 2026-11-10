# TUI Widget Gallery

Choose a widget from the list on the left and try it in the preview on the
right. Below 42 columns, the gallery shows one pane with Previous and Next
buttons. Below 32 columns or 12 rows, it asks for more space. Use the theme
buttons below the preview to switch between Light, Dark, Green, Blue and Amber.
The selected page and its values stay in place as you resize or change themes.

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

The status line shows the current page and keyboard hints. Shrink the window
to see the less important hints disappear. Open the overlay to see its
Escape hint.

| Page | Try it |
| --- | --- |
| Buttons | Increment with the primary button or clear the count with the destructive button. Compare focus and hover with the disabled button and the active theme choice. |
| Checkbox | Toggle an option and inspect a disabled checked option. |
| Text input | Check the placeholder in different themes. Type or paste text, then clear the field to see it again. |
| Lists | Select a row with the mouse or arrows and open it with Enter. Cedar cannot be opened. Scroll through the rows or toggle the empty state. |
| Progress bar | Advance from 0 to 100 percent and reset. |
| Text & layout | Resize to compare fixed and growing children. See titled and plain dividers above the row and border examples. |
| Scroll view | Append events to the log. Scroll up to pause following and press End in the viewport to resume. |
| Cell canvas | Resize the viewport and try clicks, middle clicks and the wheel. |
| Overlay | Scroll the notes with the wheel or Tab into the body and use Page Down. Resize and check that Close stays in the footer. |
| Tabs | Switch with Left and Right or the mouse. Tab enters the panel. Ctrl+T returns to General. Try it from the Appearance checkbox to see focus return to the bar. |
| Table | Resize the world catalog to see the name column adapt. Scroll the rows under the header, then select a world and open it with Enter. |
| Multiline editor | Paste several lines, then switch between Notes and Draft. Each document retains its caret and selection; the host manages its viewport. Only text edits change its status to Edited. |

The gallery uses the same model and views in the
[private debug terminal](../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
and in a headless session. For manual headless stepping, launch it with
`--headless-control` and set `FPAS_TUI_CONTROL_PIPE` to your controller's
current-user pipe named `fpas-tui-<32 hex digits>`. The session starts at 80 × 24
cells. Use the pipe protocol documented in the
[filter list example](../filter-list/README.md).

For example, activate `pages/lists` and send `focus` for `sample-list`.
Send Down and step after each action. The snapshot reports `cedar` as the value of
`sample-list`. Activate `theme-blue` and step again to see the same selection
in the Blue theme. Add new previews in `Previews.fpas` and register their pages
in `State.fpas`.
