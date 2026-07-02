# Std.Tui terminal checklist

Use this checklist after changes to `Std.Tui` session, Turbo Vision bridge, or test behavior.

| Scope | Command | Expected result |
| --- | --- | --- |
| Rust sema TUI tests | `cargo test -p fpas-sema std_units::tui` | TUI symbols typecheck. |
| Rust compiler TUI tests | `cargo test -p fpas-compiler std_library::tui` | Current TUI lowering and runtime tests pass. |
| Rust VM TUI doc links | `cargo test -p fpas-vm tui_spec_links` | TUI Rust sources link to existing `docs/pascal/std/tui/` files. |
| FPAS Turbo Vision controls | `cargo run -q -p fpas-cli -- test tests/tui/controls/` | Headless Turbo Vision widget tests pass. |
| Full FPAS suite | `cargo run -q -p fpas-cli -- test tests/` | Full regression suite passes. |
| Full Rust suite | `cargo test --workspace` | Workspace tests pass. |
| FPAS formatting | `cargo run -q -p fpas-cli -- fmt --check tests/ examples/ apps/` | No formatting drift. |

Turbo Vision regression tests under `tests/tui/controls/` (30 files). Core coverage:

- **Spike / run** — `spike_test`, `run_test`
- **Widgets** — `window`, `static_text`, `memo`, `text_viewer`, `input_line`, `list_box`, `check_box`, `check_box_mouse`, `radio_button`, `radio_button_mouse`
- **Chrome** — `chrome_test`, `menu_test`
- **Modals** — `file_dialog_test`, `exec_dialog_test`, `checked_test`
- **Live reconcile** — `live_tree_test`, `live_dialog_test`
- **Runtime setters** — `set_text`, `set_checked`, `set_items`, `set_title`, `set_menus`, `set_status_items`
- **Command map** — `reserved_command_test`

Interactive examples (manual terminal): `examples/pascal/tui/exec_dialog.fpas`, `runtime_setters.fpas`.

## See Also

- [Application](app/README.md)
- [Native testing](app/testing.md)
- [Terminal checklist](terminal-checklist.md)
