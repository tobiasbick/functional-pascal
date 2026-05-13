# TUI terminal verification checklist

Manual terminal checks for the hosted `Std.Tui` dispatch loop.

Use this checklist when changing the hosted event loop, `TuiSession`, console rendering, focus traversal, modal routing, view paint, command maps, or terminal setup/teardown. The canonical user-facing API spec is [`docs/pascal/std/tui-app.md`](../pascal/std/tui-app.md).

## Prerequisites

- Build the CLI first: `cargo build -p fpas-cli`.
- Run checks in a real terminal, not only through editor task output.
- Start each example from the repository root with `cargo run -p fpas-cli -- <example>`.
- Record the terminal, OS, shell, and terminal size for any failed or suspicious result.

## Smoke checks

| Check | Command | Expected result |
| ----- | ------- | --------------- |
| Hosted startup and shutdown | `cargo run -p fpas-cli -- examples/pascal/tui/minimal_application.fpas` | Alternate-screen/raw-mode state is restored after Escape exits. Initial paint shows the current terminal size. |
| Redraw and resize | `cargo run -p fpas-cli -- examples/pascal/tui/minimal_application.fpas` | Resizing the terminal updates the displayed size and does not leave stale rows or columns behind. |
| Local view paint | `cargo run -p fpas-cli -- examples/pascal/tui/local_view_paint.fpas` | View-local paint runs in the documented order and only redraws the visible host-managed regions. |
| View-scoped commands | `cargo run -p fpas-cli -- examples/pascal/tui/view_scoped_commands.fpas` | Tab traversal changes focus, focused-view shortcuts win over less-local command maps, and global shortcuts still work when no local binding matches. |
| Existing-view modal | `cargo run -p fpas-cli -- examples/pascal/tui/show_modal_existing_view.fpas` | While the modal is active, focus, key commands, and mouse events are scoped to the modal view subtree. Closing the modal restores background interaction. |
| Owned dialog modal | `cargo run -p fpas-cli -- examples/pascal/tui/show_dialog.fpas` | The dialog opens as an owned modal root, modal-local Escape closes it, and closing unregisters the owned root view. |

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

## Scripted-test candidates

Automated coverage should prefer host-level tests before relying on a pseudo-terminal harness. Add scripted terminal tests only for behavior that cannot be validated through the existing VM and std test surfaces.

Good candidates:

1. Startup resize coalescing before first paint.
2. Resize bursts followed by key dispatch.
3. Rapid Tab / Shift+Tab traversal with dirty-rectangle production.
4. Modal command blocking while focus is outside the modal scope.
5. Shutdown during `OnPaint` or `OnIdle` still dispatches `OnExit` with `ExitReason.HostShutdown`.

Avoid terminal-script assertions that depend on exact escape sequences unless the test owns the backend abstraction being checked.