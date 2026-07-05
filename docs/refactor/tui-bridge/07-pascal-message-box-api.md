# 07 — Optional public `Std.Tui` message box API

**Status:** [ ] Not started · [ ] In progress · [ ] Done

**Priority:** Low — only when FPAS programs need standard dialogs beyond IDE internals

**Depends on:** [02-about-message-box.md](02-about-message-box.md) (Rust helper proven), [done/02-single-tv-session.md](done/02-single-tv-session.md)

## Problem

Today only widget-composition APIs exist (`CreateDialog`, `ExecDialog`, …). Common Turbo Vision flows use `message_box` / `input_box` helpers. Without a thin Pascal surface, every app rebuilds OK/Cancel dialogs by hand (as IDE About did before [02](02-about-message-box.md)).

## Target (if implemented)

Minimal Pascal API wrapping upstream helpers, not reimplemented layout:

```pascal
{ sketch — names and shapes TBD }
Application.MessageBox(App, Message, Flags): integer;
Application.InputBox(App, Title, Label, Default, Limit): DialogResult + string;
```

Flags could mirror upstream `MF_*` constants as `MessageBox.About`, `MessageBox.OkButton`, … or a single options integer documented beside Borland.

## Tasks

- [ ] **Need** — Confirm at least two call sites (IDE + one example/test) want public API vs internal-only Rust helper.
- [ ] **Spec** — Page under `docs/pascal/std/tui/app/` (e.g. `message-box.md`); link from app README.
- [ ] **Sema/registry** — Register symbols in `fpas-sema` `std_registry/loaded/tui/`.
- [ ] **Compiler/bytecode/vm** — Intrinsics calling same Rust helper as [02](02-about-message-box.md).
- [ ] **Migrate IDE** — `ShowAbout` uses public API if exposed.
- [ ] **Example** — Short `examples/pascal/tui/message_box.fpas` (not `*_test.fpas`).
- [ ] **Tests** — Headless close command via `TestSetDialogResult` or MockTerminal path.
- [ ] **Mark done** — Check boxes in this file; update [README.md](../README.md) status table.

## Non-goals

- Full `MF_*` combinatorial surface on day one — start with About + OK + OK/Cancel
- Mirroring every upstream helper (`inputBoxRect`, …) without a caller

## Files (expected touch)

```text
docs/pascal/std/tui/app/message-box.md
crates/fpas-sema/src/std_registry/loaded/tui/
crates/fpas-compiler/src/compiler/std_calls/tui/
crates/fpas-bytecode/src/intrinsic/tui/
crates/fpas-vm/src/vm/execute/io/tui/msgbox.rs
examples/pascal/tui/
tests/tui/controls/   (optional *_test.fpas)
```

## Verification

```text
cargo fmt && cargo build && cargo test --workspace
fpas fmt --check examples/pascal/tui/ tests/tui/
cargo run -q -p fpas-cli -- test tests/tui/controls/   {if test added}
```

## Notes

- Prefer upstream `MF_ABOUT`, `MF_OK_BUTTON`, `CM_OK` mapping through existing `Command.Accept`.
- If YAGNI wins, close this item as “won’t do” and keep IDE on internal helper only — document that decision at top.
