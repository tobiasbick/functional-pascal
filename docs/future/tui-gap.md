# CLR TUI gaps

The current host, element and control contracts are documented with
[Fpas.Tui](../../src/Fpas.Tui/README.md), the
[filter-list example](../../examples/tui/filter-list/README.md) and
[private-terminal debugging](../pascal/tools/debugging.md#private-terminal-and-tui-control).

## Follow-up elements

Tabs, menus, tables and a multiline editor.

## Application-owned theme preferences

Applications that offer persistent preferences choose how to load and save
their selected preset or custom RGB colors. The
[theme interface](../../src/Fpas.Tui/README.md#styling) supplies Light, Dark,
Green and Blue and applies app-defined palette changes during a running session.
