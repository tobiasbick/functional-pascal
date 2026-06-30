# Std.Tui terminal checklist

Use this checklist after changes to `Std.Tui` session, backend, or test behavior.

| Scope | Command | Expected result |
| --- | --- | --- |
| Rust sema TUI tests | `cargo test -p fpas-sema std_units::tui` | Current TUI symbols typecheck and removed symbols fail. |
| Rust compiler TUI tests | `cargo test -p fpas-compiler std_library::tui` | Current TUI lowering and runtime tests pass. |
| FPAS TUI tests | `cargo run -q -p fpas-cli -- test tests/tui/` | Headless TUI regression tests pass. |
| Full Rust suite | `cargo test --workspace` | Workspace tests pass. |

## See Also

- [Application](app/README.md)
- [Native testing](app/testing.md)
