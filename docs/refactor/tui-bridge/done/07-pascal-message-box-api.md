# Done — Public `Std.Tui` message box API

**Status:** [x] Done (2026-07)

**Depends on:** [done/03-about-message-box.md](03-about-message-box.md)

## Summary

Public spec at [docs/pascal/std/tui/app/message-box.md](../../pascal/std/tui/app/message-box.md). `MessageBoxOption.*` constants registered in sema/compiler. IDE About uses named flags. Example and headless regression test added.

## Completed tasks

- [x] **Spec** — `message-box.md`; links from app README, modals, types, handlers
- [x] **Constants** — `fpas-std` `message_box_options.rs`, sema `message_box_api.rs`, compiler `builtin_consts.rs`
- [x] **IDE** — `about.fpas` uses `MessageBoxOption.About + MessageBoxOption.OkButton`
- [x] **Example** — `examples/pascal/tui/message_box.fpas`
- [x] **Tests** — `tests/tui/controls/tui_turbo_vision_message_box_test.fpas`; IDE tests unchanged

## Verification

```text
cargo fmt && cargo build && cargo test --workspace
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_message_box_test.fpas
cargo run -q -p fpas-cli -- test apps/ide/tests/
```

## Notes

- `Application.InputBox` remains out of scope (no caller yet).
- Combine option flags with `+` in Pascal.
