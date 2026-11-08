# CLR TUI gaps

The current host, element and control contracts are documented with
[Fpas.Tui](../../src/Fpas.Tui/README.md), the
[filter-list example](../../examples/tui/filter-list/README.md) and
[private-terminal debugging](../pascal/tools/debugging.md#private-terminal-and-tui-control).

## Widgets

1. **Multiline editor.** Preserve line breaks on paste and keep the caret
   visible as it moves. Let the app restore the caret and scroll position
   when it switches documents. Report text edits separately from caret and
   scroll changes so navigation leaves the document's dirty state alone.
2. **Overlay sizing.** Set size limits relative to the available terminal
   space. Give long content a scrollable body and reserve a footer so
   actions stay visible after a resize.
3. **Tabs.** Show the active tab and switch views by keyboard or mouse.
   Define where focus goes when the visible view changes.
4. **Table.** Build on the selection list. Start with a header, fixed and
   flexible column widths, and selectable rows.

Add placeholders to single-line inputs, dividers with optional titles, a
status line for messages and shortcut hints, and reusable styles for active,
primary and destructive actions.

## Host

- Give application views the initial terminal dimensions and the new size
  on resize. Views can use them to switch between two panes and one pane or
  to show a fallback when the terminal is too small.
- Add a focus command that targets a stable control ID in the resulting
  view. Keep it within the active focus scope. A shortcut can then move
  focus to search, or activating an item can open and focus its editor.

## Application-owned theme preferences

Applications that offer persistent preferences choose how to load and save
their selected preset or custom RGB colors. The
[theme interface](../../src/Fpas.Tui/README.md#styling) supplies Light, Dark,
Green and Blue and applies app-defined palette changes during a running session.
