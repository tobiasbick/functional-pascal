# CLR diagnostic acceptance

The [diagnostic reference](../pascal/tools/diagnostics.md) owns the shared payload,
CLI text/JSON presentation, MSBuild forwarding and bounded recovery behavior.

## Visual Studio presentation

Verify actual Visual Studio error-list display, start/end navigation, warning
severity and help expansion on Windows. Check that explanation and related-note
messages do not create extra error-list entries. SDK integration tests verify
the emitted MSBuild events, but cannot establish the IDE's presentation.

Editor actions and quick fixes remain in [editor gaps](editor-gap.md).
MSBuild logging does not apply proposed changes; explicit CLI application uses
[check --fix](../pascal/tools/diagnostics.md#apply-automatic-corrections).
