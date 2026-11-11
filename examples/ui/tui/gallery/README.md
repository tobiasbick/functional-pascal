# FPas CUI TUI gallery

After the [local SDK bootstrap](../../../../docs/build/development.md#build-an-fpas-application),
build and run the gallery from the repository root.

```sh
dotnet build examples/ui/tui/gallery/TuiWidgetGallery.fpasproj
dotnet examples/ui/tui/gallery/bin/Debug/net10.0/TuiWidgetGallery.dll
```

The terminal version adapts the CUI gallery to cells. It has the same four
pages and shows the widgets checked in the [CUI inventory](../../../../docs/future/cui-gaps.md).

- **Overview** introduces the gallery and links to Buttons.
- **Buttons** shows variants, sizes, icons, states, the formatting group, copy,
  and hold to confirm. Copy uses OSC 52 in terminals that allow clipboard writes.
  To confirm a hold, keep the left mouse button down until the fill reaches
  the end, then release over the same button.
- **Overlays** opens the Notifications sheet.
- **Actions** shows Save and Export split buttons with anchored menus and an
  inline result. Tab focuses each half. Down opens the chevron's menu, Up and
  Down select an item, Enter chooses, and Escape or an outside click closes it.
  Delete project needs two activations within three seconds and shows the
  remaining seconds. Escape or a click outside cancels; confirmation only
  increments the harmless preview counter.
  Save changes shows inline work feedback with result colors. Complete, Fail
  and Reset drive the model; non-idle feedback retains focus and rejects activation.
  The Theme section and navigation use the same light/dark/System switcher.
  Tab moves focus, Left/Right select without wrapping, and Enter or Space
  selects the focused segment. The source button simulates an application
  appearance-provider change; System itself does not detect terminal settings.
  The adjacent sun/moon Theme toggle shows its target and proposes an explicit
  opposite light/dark choice, including when System was selected.

Use the pointer or Tab and Enter to choose a page. Page Up and Page Down switch
pages when the content viewport does not own the keys. Below 56 columns, use
the **Components** icon to open the left drawer. Escape closes the drawer or
sheet. Use Ctrl+Q to exit.

The gallery supports the [private debug terminal](../../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control).
For headless stepping, launch it with `--headless-control` and set
`FPAS_TUI_CONTROL_PIPE` to the controller's current-user pipe named
`fpas-tui-<32 hex digits>`. The initial frame is 80 × 24 cells.
