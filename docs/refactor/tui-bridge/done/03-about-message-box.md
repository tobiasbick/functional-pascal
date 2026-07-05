# Done — About and simple dialogs via upstream `message_box`

**Status:** [x] Done (2026-07)

**Depends on:** [02-single-tv-session.md](02-single-tv-session.md)

**Follow-up:** [done/07-pascal-message-box-api.md](07-pascal-message-box-api.md) (public Pascal spec)

## Summary

IDE Help → About calls `Application.MessageBox` on the shared live turbo-vision session. The VM bridge invokes upstream `turbo_vision::helpers::msgbox::message_box` from `msgbox.rs`. Headless tests reuse `Application.TestSetDialogResult` (same queue as `ExecDialog`).

## Completed tasks

- [x] **VM** — `msgbox.rs` + `TuiIntrinsic::MessageBox` dispatch
- [x] **Stack** — bytecode `widgets.inc`, sema, compiler, `STD_TUI_APPLICATION_MESSAGE_BOX`
- [x] **IDE** — `about.fpas` shrinks to `MessageBox` + `AboutMessage()` (`1028` = `MF_ABOUT | MF_OK_BUTTON`)
- [x] **Tests** — `apps/ide/tests/` (6/6) unchanged assertions; `cargo test --workspace`
- [x] **Docs** — `vm-bridge.md`, `modals.md`, `handlers.md`, `00-context.md`, refactor README

## Files touched

```text
crates/fpas-vm/src/vm/execute/io/tui/msgbox.rs
crates/fpas-vm/src/vm/execute/io/tui/mod.rs
crates/fpas-bytecode/src/intrinsic/tui/variants/widgets.inc
crates/fpas-compiler/src/compiler/std_calls/tui/application.rs
crates/fpas-sema/src/std_registry/loaded/tui/application_api.rs
crates/fpas-std/src/std_units/symbols/std_symbols/tui.rs
apps/ide/src/dialog/about.fpas
docs/pascal/std/tui/app/vm-bridge.md
```

## Verification

```text
cargo test --workspace
cargo run -q -p fpas-cli -- test apps/ide/tests/
```

## Notes

- `Application.MessageBox` is documented in [docs/pascal/std/tui/app/message-box.md](../../pascal/std/tui/app/message-box.md).
- Custom dialogs with read-back still use `CreateDialog` + `ExecDialog`.
