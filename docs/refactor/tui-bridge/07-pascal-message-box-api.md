# 07 — Optional public `Std.Tui` message box API

**Status:** [ ] Not started · [ ] In progress · [ ] Done

**Priority:** Low — only when FPAS programs need standard dialogs beyond IDE internals

**Depends on:** [done/03-about-message-box.md](done/03-about-message-box.md) (Rust helper proven), [done/02-single-tv-session.md](done/02-single-tv-session.md)

## Problem

Today only widget-composition APIs exist for custom dialogs (`CreateDialog`, `ExecDialog`, …). `Application.MessageBox` is registered for IDE use ([done/03](done/03-about-message-box.md)) but has no public spec page yet. Common Turbo Vision flows also include `input_box` helpers.

## Target (if implemented)

Minimal Pascal API wrapping upstream helpers, not reimplemented layout:

```pascal
{ sketch — names and shapes TBD; phase 1 may ship Application.MessageBox for IDE only without this doc page }
Application.MessageBox(App, Message, Flags): integer;
Application.InputBox(App, Title, Label, Default, Limit): DialogResult + string;
```

Phase 1 shipped `Application.MessageBox` in sema/stdlib for IDE internal use ([done/03-about-message-box.md](done/03-about-message-box.md)); this item adds the public spec and named constants.

Upstream flag values at turbo-vision 2.0.0 (see `helpers/msgbox.rs`): type `MF_ABOUT = 4`, `MF_OK_BUTTON = 0x0400`; closing `CM_OK = 10` matches `Command.Accept`. Implementation: [done/03-about-message-box.md](done/03-about-message-box.md).

## Tasks

- [ ] **Need** — Confirm at least two call sites (IDE + one example/test) want public API vs internal-only Rust helper.
- [ ] **Spec** — Page under `docs/pascal/std/tui/app/` (e.g. `message-box.md`); link from app README.
- [ ] **Sema/registry** — Register symbols in `fpas-sema` `std_registry/loaded/tui/`.
- [ ] **Compiler/bytecode/vm** — Already implemented in [done/03](done/03-about-message-box.md); extend only if public API adds symbols.
- [ ] **Migrate IDE** — `ShowAbout` uses public API if exposed.
- [ ] **Example** — Short `examples/pascal/tui/message_box.fpas` (not `*_test.fpas`).
- [ ] **Tests** — Headless close command via `TestSetDialogResult` or MockTerminal path.
- [ ] **Mark done** — Check boxes in this file; update [README.md](../README.md) status table.

## Non-goals

- Full `MF_*` combinatorial surface on day one — start with About + OK + OK/Cancel
- Mirroring every upstream helper (`inputBoxRect`, …) without a caller

## Files (expected touch)

```text
crates/fpas-sema/src/std_registry/loaded/tui/application_api.rs
crates/fpas-compiler/src/compiler/std_calls/tui/application.rs
crates/fpas-bytecode/src/intrinsic/tui/variants/session.inc
crates/fpas-std/src/std_units/symbols/std_symbols/tui.rs
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
