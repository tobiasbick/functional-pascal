# Completed TUI Implementation

This file records what is already implemented so new plans do not reopen finished work.

## Hosted Application Framework

Implemented:

- `Application.Run`, `Application.Configure`, `ApplicationHandlers`, `ExitReason`, `OnIdle`,
  `OnExit`, `OnKeyPressed`, `OnMouse`, `OnPaste`, `OnFocusGained`, `OnFocusLost`, and `OnResize`.
- Shared host event normalization, resize coalescing, buffered redraw, damage tracking, and
  headless test hosts.
- VM bridge, compiler lowering, sema registration, and diagnostics for the hosted API.
- Poll-style terminal input remains on `Std.Console`; full applications use `Std.Tui`.

Evidence:

- Spec: [`docs/pascal/std/tui/app/README.md`](../../pascal/std/tui/app/README.md)
- Handlers: [`docs/pascal/std/tui/app/handlers.md`](../../pascal/std/tui/app/handlers.md)
- Tests: `crates/fpas-vm/src/tests/core/tui_host_vm/`, `tests/tui/host/`

## Retained Views and Routing

Implemented:

- Parent-relative view tree, z-order, focused leaf/root state, modal scope, local `OnViewPaint`,
  retained clipping, pointer capture, sourced commands, and typed process outcomes.
- `ViewState`, `ViewOptions`, `ViewLayout`, `HostSetViewLayout`, and `QueryViewLayout`.
- Indexed `ViewRegistry` lookup while preserving explicit root/child order vectors.

Evidence:

- Spec: [`docs/pascal/std/tui/app/views.md`](../../pascal/std/tui/app/views.md)
- Tests: `tests/tui/scene/tui_view_clip_test.fpas`,
  `tests/tui/scene/tui_view_layout_test.fpas`

## Frames, Windows, and Dialogs

Implemented:

- `HostCreateFrameView`, `ShowFramedDialog`, frame chrome, frame viewport clipping, active/inactive
  styling, close, zoom/restore, move, resize, next-window activation, cascade, tile, window list,
  and frame scroll chrome.
- Owned modal-root cleanup, nested focus restore, default/cancel commands, and modal results.
- Reserved sourced frame commands replaced the earlier `FrameAction` callback idea.

Evidence:

- Spec: [`docs/pascal/std/tui/app/frames.md`](../../pascal/std/tui/app/frames.md)
- Tests: `tests/tui/frames/`, `tests/tui/modals/tui_framed_dialog_test.fpas`,
  `tests/tui/modals/tui_show_framed_dialog_controls_test.fpas`
- Examples: `examples/pascal/tui/framed_window.fpas`, `examples/pascal/tui/framed_dialog.fpas`,
  `examples/pascal/tui/show_dialog.fpas`, `apps/ide/src/dialog.fpas`

## Controls, Layout, and Cell Width

Implemented:

- Labels, buttons, input line, checkbox, radio group, list box, standalone scroll bar, scroll view,
  memo, shared scroll model, and shared scroll-thumb geometry.
- Unicode display-width policy for console paint, frame titles, labels, buttons, list rows, input
  cursor/scroll placement, and memo cursor placement.

Evidence:

- Spec: [`docs/pascal/std/tui/app/controls.md`](../../pascal/std/tui/app/controls.md)
- Cell width: [`docs/pascal/std/tui/cell-width.md`](../../pascal/std/tui/cell-width.md)
- Tests: `tests/tui/controls/`

## Representative Regression Tests

- `tests/tui/frames/tui_frame_chrome_test.fpas`
- `tests/tui/frames/tui_frame_chrome_actions_test.fpas`
- `tests/tui/frames/tui_frame_occlusion_test.fpas`
- `tests/tui/frames/tui_frame_occlusion_move_test.fpas`
- `tests/tui/frames/tui_frame_occlusion_resize_test.fpas`
- `tests/tui/frames/tui_frame_occlusion_zoom_test.fpas`
- `tests/tui/frames/tui_frame_reserved_commands_test.fpas`
- `tests/tui/frames/tui_frame_scroll_test.fpas`
- `tests/tui/frames/tui_frame_scroll_clip_test.fpas`
- `tests/tui/frames/tui_frame_scroll_input_clip_test.fpas`
- `tests/tui/frames/tui_frame_window_list_test.fpas`
- `tests/tui/modals/tui_framed_dialog_test.fpas`
- `tests/tui/modals/tui_show_framed_dialog_controls_test.fpas`
- `tests/tui/controls/tui_controls_test.fpas`
- `tests/tui/controls/tui_memo_test.fpas`
- `tests/tui/controls/tui_cell_width_test.fpas`
- `tests/tui/scene/tui_view_clip_test.fpas`
- `tests/tui/scene/tui_view_layout_test.fpas`
- `tests/tui/menu/tui_menu_overlay_frame_test.fpas`

## Implementation Pointers

- `crates/fpas-std/src/tui/view/`
- `crates/fpas-std/src/tui/widget/frame/`
- `crates/fpas-std/src/tui/widget/control/`
- `crates/fpas-std/src/tui/scroll/`
- `crates/fpas-std/src/tui/modal/`
- `crates/fpas-vm/src/vm/execute/io/tui/`
