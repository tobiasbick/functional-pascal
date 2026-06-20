# TUI terminal verification checklist

Manual terminal checks for hosted `Std.Tui` applications.

Use this checklist after changes to terminal behavior, hosted dispatch, focus, modals, or view paint. Headless regression tests run under `fpas test` — see [Native testing](app/testing.md). API reference: [Hosted dispatch](app/README.md).

## Prerequisites

- Build or install `fpas` first (`cargo build --release -p fpas-cli`).
- Run checks in a real terminal, not only through editor task output.
- Start each example from the repository root with `fpas <path>`.
- Record the terminal, OS, shell, and terminal size for any failed or suspicious result.

## Smoke checks

| Check | Command | Expected result |
| ----- | ------- | --------------- |
| Hosted startup and shutdown | `fpas examples/pascal/tui/minimal_application.fpas` | Alternate-screen/raw-mode state is restored after Escape exits. Initial paint shows the current terminal size. |
| Redraw and resize | `fpas examples/pascal/tui/minimal_application.fpas` | Resizing the terminal updates the displayed size and does not leave stale rows or columns behind. |
| Local view paint | `fpas examples/pascal/tui/local_view_paint.fpas` | View-local paint runs in the documented order and only redraws the visible host-managed regions. |
| View-scoped commands | `fpas examples/pascal/tui/view_scoped_commands.fpas` | Tab traversal changes focus, focused-view shortcuts win over less-local command maps, and global shortcuts still work when no local binding matches. |
| Existing-view modal | `fpas examples/pascal/tui/show_modal_existing_view.fpas` | While the modal is active, focus, key commands, and mouse events are scoped to the modal view subtree. Closing the modal restores background interaction. |
| Owned dialog modal | `fpas examples/pascal/tui/show_dialog.fpas` | Opens an owned modal root with OK/Cancel focus targets. Escape sets Cancel via modal-local command binding, closes the dialog, unregisters the owned root, and restores background focus. |
| IDE About dialog | `fpas apps/ide/ide.fpasprj` | Help → About opens a modal dialog; Enter on OK or Escape closes it and restores the shell. |

## Real-terminal observations

For each smoke check, verify these points before marking the run clean:

1. The terminal mode is restored after normal quit and after closing the terminal window.
2. The cursor is not left hidden after the program exits.
3. The screen does not flicker excessively during resize, focus traversal, or modal open/close.
4. The first frame appears without requiring a key press.
5. Rapid resize bursts settle on the final terminal size.
6. Escape or the documented quit shortcut exits exactly once and does not require a second key press.
7. Mouse-enabled examples do not leak clicks from an active modal to background views.
8. Paste/focus events, where supported by the terminal, do not panic and do not starve paint or quit handling.

## See also

- [Native testing](app/testing.md)
- [Hosted dispatch](app/README.md)
- [Terminal UI index](README.md)
- [Standard library index](../README.md)
