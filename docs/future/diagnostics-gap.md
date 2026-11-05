# CLR diagnostic acceptance

The [diagnostic reference](../pascal/tools/diagnostics.md) owns the shared payload,
CLI text/JSON presentation, MSBuild forwarding and bounded recovery behavior.

## Visual Studio presentation

Verify actual Visual Studio error-list display, start/end navigation, warning
severity and help expansion on Windows. Check that explanation and related-note
messages do not create extra error-list entries. SDK integration tests verify
the emitted MSBuild events, but cannot establish the IDE's presentation.

Automatic edit application remains in [CLI gaps](cli-gap.md); editor actions and
quick fixes remain in [editor gaps](editor-gap.md). Neither compiler diagnostics
nor MSBuild logging applies proposed changes.
