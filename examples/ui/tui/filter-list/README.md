# Filter list

The example uses the same FPAS model, update function and view in an interactive
terminal and a manually stepped headless session. The example uses the
`Fpas.Ui.Themes.Dark()` CUI palette by default. Type in the search field or use
Ctrl+O to open the Options overlay. Select Light, Dark, Green, Blue or Amber to
recolor the current screen immediately. Ctrl+R refreshes the main screen or
resets the options to their defaults while the overlay is open. Ctrl+Q exits
the main screen. Escape closes the overlay.

Build with the current local FPAS SDK. [Development workflow](../../../../docs/build/development.md#build-an-fpas-application)

```sh
dotnet build examples/ui/tui/filter-list/FilterList.fpasproj
dotnet examples/ui/tui/filter-list/bin/Debug/net10.0/FilterList.dll
dotnet examples/ui/tui/filter-list/bin/Debug/net10.0/FilterList.dll --headless
```

The headless run checks filtering, focus restoration, live theme switching,
background progress and resize and prints the completed frame. For external manual stepping, start
with `--headless-control` and set `FPAS_TUI_CONTROL_PIPE` to the controller's
current-user pipe name `fpas-tui-<32 hex digits>`. The control stream uses
four-byte big-endian lengths and UTF-8 JSON. Send requests with a positive
`id` and an `operation` of `snapshot`, `send`, `wait`, `resize` or `step`.
The session emits versioned `frame` events and correlates each reply by ID.

Use the private-terminal DAP launch to inspect the interactive session at a
source breakpoint. [TUI debug requests](../../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
