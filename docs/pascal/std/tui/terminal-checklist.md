# Std.Tui terminal checklist

Use this checklist after changes to `Std.Tui` session, Turbo Vision bridge, or test behavior.

| Scope | Command | Expected result |
| --- | --- | --- |
| Rust sema TUI tests | `cargo test -p fpas-sema std_units::tui` | Current TUI symbols typecheck and removed symbols fail. |
| Rust compiler TUI tests | `cargo test -p fpas-compiler std_library::tui` | Current TUI lowering and runtime tests pass. |
| Rust VM TUI doc links | `cargo test -p fpas-vm tui_spec_links` | TUI Rust sources link to existing `docs/pascal/std/tui/` files. |
| FPAS Turbo Vision controls | `cargo run -q -p fpas-cli -- test tests/tui/controls/` | Headless Turbo Vision widget tests pass. |
| FPAS hosted transition tests | `cargo run -q -p fpas-cli -- test tests/tui/host/` | Headless hosted-loop and screen-query tests pass. |
| Full FPAS suite | `cargo run -q -p fpas-cli -- test tests/` | Full regression suite passes. |
| Full Rust suite | `cargo test --workspace` | Workspace tests pass. |
| FPAS formatting | `cargo run -q -p fpas-cli -- fmt --check tests/ examples/ apps/` | No formatting drift. |

Turbo Vision regression tests under `tests/tui/controls/`:

- `tui_turbo_vision_spike_test.fpas` — dialog, button, `OnCommand`
- `tui_turbo_vision_run_test.fpas` — `Application.Run` path
- `tui_turbo_vision_window_test.fpas` — `Window`, `AddWindow`
- `tui_turbo_vision_static_text_test.fpas`
- `tui_turbo_vision_memo_test.fpas`
- `tui_turbo_vision_input_line_test.fpas`
- `tui_turbo_vision_list_box_test.fpas`
- `tui_turbo_vision_check_box_test.fpas`
- `tui_turbo_vision_radio_button_test.fpas`
- `tui_turbo_vision_chrome_test.fpas` — menu bar and status line
- `tui_turbo_vision_file_dialog_test.fpas` — `RunFileDialog`, `TestSetFileDialogResult`
- `tui_turbo_vision_exec_dialog_test.fpas` — `ExecDialog`, `InputText`, `TestSetDialogResult`
- `tui_turbo_vision_text_viewer_test.fpas` — `TextViewer`, `AddChild`

## See Also

- [Application](app/README.md)
- [Native testing](app/testing.md)
- [Future testing plan](../../../future/turbo-vision-4-rust/05-testing-plan.md)
