# TUI platform acceptance

The current terminal contract is in [Fpas.Ui.Tui](../../../src/Fpas.Ui.Tui/README.md),
using [Fpas.Ui](../../../src/Fpas.Ui/README.md) views and
[Fpas.Ui.Mvu](../../../src/Fpas.Ui.Mvu/README.md) transitions. Automated ownership and
shared-view checks are listed in the [acceptance map](../../../tests/coverage.md#terminal-ui).

Phase 7 requires native Windows x64 acceptance in addition to Linux validation.

- [ ] Run HeadlessTuiTests, TerminalModeTests, TerminalBoundaryTests,
  WidgetContractTests and the TUI/gallery/Mandelbrot DAP scenarios on Windows.
  Verify terminal input modes are restored after both success and failure.
- [ ] Run GuiHostTests with software and OpenGL rendering on Windows. Its portable
  fixture must produce the same messages, controlled values, selection and modal
  focus semantics under TUI and GUI.
- [ ] Build and run the terminal examples without CUI assets or GUI build tools
  on Windows. Verify cell width, resizing, scoped shortcuts and editor selection
  through the real terminal event path.
- [ ] Resolve Windows findings before closing phase 7. Final cross-platform
  publishing and documentation acceptance belongs to
  [phase 8](verification-and-documentation.md).
