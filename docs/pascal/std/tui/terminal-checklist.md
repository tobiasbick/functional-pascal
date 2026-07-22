# Std.Tui terminal checklist

Use this checklist after changes to `Std.Tui` session, Turbo Vision bridge, or test behavior.

| Scope | Command | Expected result |
| --- | --- | --- |
| Rust sema TUI tests | `cargo test -p fpas-sema std_units::tui` | TUI symbols typecheck. |
| Rust compiler TUI tests | `cargo test -p fpas-compiler std_library::tui` | Current TUI lowering and runtime tests pass. |
| Rust VM TUI doc links | `cargo test -p fpas-vm tui_spec_links` | TUI Rust sources link to existing `docs/pascal/std/tui/` files. |
| FPAS Turbo Vision tests | `cargo run -q -p fpas-cli -- test tests/tui/` | Headless and smoke tests pass. |
| Full FPAS suite | `cargo run -q -p fpas-cli -- test tests/` | Full regression suite passes. |
| Full Rust suite | `cargo test --workspace` | Workspace tests pass. |
| FPAS formatting | `cargo run -q -p fpas-cli -- fmt --check tests/ examples/ apps/` | No formatting drift. |

Turbo Vision bridge regression tests are under `tests/tui/views/`, `tests/tui/smoke/`, `tests/tui/modals/`, and `tests/tui/events/`. The retained `apps/ide/` source is legacy and is not tested.

Core coverage:

- **Run / quit** — `run_quit_test`, `window_quit_test`
- **Widgets** — `tests/tui/views/*_test.fpas`
- **Chrome** — `window_chrome_test`, `menu_bar_set_menus_test`, `status_line_set_items_test`
- **Modals** — `message_box_test`, `file_dialog_test`
- **Events** — `on_key_test`, `on_mouse_test`, mouse smoke tests
- **Command ids** — `reserved_command_test`

Interactive examples (manual terminal): `examples/pascal/tui/turbo_vision_dialog.fpas` and `turbo_vision_window.fpas`. The retained `apps/ide/` source is not executable or tested.

## See Also

- [Application](app/README.md)
- [Native testing](app/testing.md)
- [VM bridge](app/vm-bridge.md)
