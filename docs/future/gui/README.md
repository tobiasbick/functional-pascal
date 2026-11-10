# Desktop GUI platform acceptance

The implemented application path is described in the [GUI handbook](../../pascal/libraries/gui.md),
[GUI host](../../../src/Fpas.Ui.Gui/README.md), [shared widgets](../../../src/Fpas.Ui/README.md)
and [terminal host](../../../src/Fpas.Ui.Tui/README.md). The
[acceptance map](../../../tests/coverage.md#native-gui) identifies automated checks.

The remaining platform is native Windows x64. Keep the phases separate so each
set of evidence can be closed independently.

| Phase | Remaining work |
| --- | --- |
| 6 | [GUI host Windows acceptance](gui-host.md) |
| 7 | [TUI Windows acceptance](tui-alignment.md) |
| 8 | [Distribution and final Windows acceptance](verification-and-documentation.md) |

Use a Windows desktop session with .NET 10, the CUI source-build prerequisites
and the .NET NativeAOT toolchain. Run the public counter and gallery with software
and OpenGL rendering. Store screenshots, logs and temporary publish outputs under
ignored `.temp-data`; keep the owning references focused on current contracts.

For upstream CUI findings, follow the [contribution policy](../../../AGENTS.md#cui-contributions).
Direct Wayland, macOS, other architectures, accessibility implementation and full
platform IME/complex-script support are outside the initial contract.

Resolve findings, remove completed checklist entries and delete this directory
when all three Windows acceptance files are closed.
