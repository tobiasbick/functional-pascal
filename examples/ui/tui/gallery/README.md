# TUI gallery

The left panel lists the GUI gallery's Components and Extended destinations plus
**Progress bar** and **Text**. The right panel previews the selected entry.
Progress bar shows a determinate track and percentage; Text shows a heading,
wrapping body text and muted caption. Other categories give a short description.
The panels share the terminal width in a 1:2 ratio with a one-cell gap.

Use ↑/↓, Page Up/Page Down or Home/End to move through the list. Enter, Space
or a mouse click opens the destination. The wheel scrolls the list. On Overview,
Feedback and Progress bar, ←/→ changes the sample by ten percentage points.
Ctrl+T switches themes and Ctrl+Q exits.

![Gallery in a 100 × 28 terminal](panels.png)

After the [local SDK bootstrap](../../../../docs/build/development.md#build-an-fpas-application),
build and run the gallery from the repository root.

```sh
dotnet build examples/ui/tui/gallery/TuiWidgetGallery.fpasproj
dotnet examples/ui/tui/gallery/bin/Debug/net10.0/TuiWidgetGallery.dll
```

The gallery supports the [private debug terminal](../../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control).
For headless stepping, launch it with `--headless-control` and set
`FPAS_TUI_CONTROL_PIPE` to the controller's current-user pipe named
`fpas-tui-<32 hex digits>`. The initial frame is 80 × 24 cells.
