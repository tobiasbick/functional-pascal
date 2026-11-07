# CLR TUI gaps

The current host, element and control contracts are documented with
[Fpas.Tui](../../src/Fpas.Tui/README.md), the
[filter-list example](../../examples/tui/filter-list/README.md) and
[private-terminal debugging](../pascal/tools/debugging.md#private-terminal-and-tui-control).

## Version 1 verification gaps

- Measure allocations and frame time for progress-only updates, resize and large
  lists. The host reuses flex allocations and painted rows and writes changed
  cell runs.
- Verify native terminal mode restoration after both normal exit and a failed
  presentation. The terminal tests exercise private-terminal input, output,
  resize and process exit. The headless fixture injects a partial write into
  the presenter and checks the full-redraw retry.
- Extend effect coverage to exercise replacement with the same ID, worker
  failure delivery and cancellation and joining on exit.

## Follow-up elements

Tabs, menus, tables and a multiline editor.
