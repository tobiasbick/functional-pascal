# 02 — About and simple dialogs via upstream `message_box`

**Status:** [ ] Not started · [ ] In progress · [ ] Done

**Priority:** Medium (quick win; session model is in place)

**Depends on:** [done/02-single-tv-session.md](done/02-single-tv-session.md) — completed. Live About during `Run` uses `Worker.live_turbo_vision_app` via `turbo_vision_with_live_app`.

**Blocks:** [07-pascal-message-box-api.md](07-pascal-message-box-api.md) (optional public API)

## Problem

`apps/ide/src/dialog/about.fpas` manually composes `CreateDialog`, `CreateTextViewer`, `CreateButton`, and `ExecDialog` to mimic Turbo Vision’s About box. Upstream already provides:

- `turbo_vision::helpers::msgbox::{message_box, message_box_ok, …}`
- `MF_ABOUT | MF_OK_BUTTON` title and layout (see upstream `msgbox.rs`)

We duplicate layout, sizing, and Borland conventions in FPAS.

## Target

- IDE About (and similar one-shot messages) call upstream `message_box` from Rust, not hand-built FPAS widget trees.
- Same Borland look as native TV examples (`menu_status.rs`, `desktop_logo.rs`).
- Pascal IDE may call a thin internal hook or a future public API; avoid large FPAS layout code for standard message boxes.

## Tasks

- [ ] **Spike** — From bridge code, call `message_box(app, text, MF_ABOUT | MF_OK_BUTTON)` on the live session via `turbo_vision_with_live_app` in `session_app.rs`.
- [ ] **Bridge helper** — Add private Rust helper (e.g. in new `msgbox.rs` under `tui/`) that takes message + flags, runs modal, returns closing `CommandId` mapped through `command_map.rs`.
- [ ] **IDE** — Replace `Ide.Dialog.About.ShowAbout` body with call to bridge helper (keep `CmdHelpAbout` menu wiring in FPAS).
- [ ] **Remove dead FPAS layout** — Delete manual dialog construction in `about.fpas` once helper works; keep `AboutMessage()` / title strings in FPAS or move to Rust constants (team choice).
- [ ] **Tests** — Update `apps/ide/tests/dialog/dialog_test.fpas` and `about_menu_test.fpas`; headless path may still use `TestSetDialogResult` until [03-headless-test-util.md](03-headless-test-util.md).
- [ ] **Docs** — Contributor note in `vm-bridge.md` when `msgbox.rs` exists; public Pascal API waits for [07](07-pascal-message-box-api.md). No change to [modals.md](../../pascal/std/tui/app/modals.md) until a public symbol is added.

## Files (expected touch)

```text
crates/fpas-vm/src/vm/execute/io/tui/msgbox.rs   (new, ~80–120 LOC)
apps/ide/src/dialog/about.fpas                   (shrink)
apps/ide/src/dialog.fpas
apps/ide/tests/…
docs/pascal/std/tui/app/vm-bridge.md             (module row when added)
```

## Verification

```text
cargo run -q -p fpas-cli -- test apps/ide/tests/
cargo run -q -p fpas-cli -- apps/ide/ide.fpasprj   # manual: Help → About
```

## Notes

- Do not reimplement `message_box` sizing logic in FPAS.
- `input_box` is a separate follow-up if needed (upstream `helpers::msgbox::input_box`).
- Standard About boxes are not built with `CreateDialog` + `ExecDialog`; custom dialogs still use the handle graph in [modals.md](../../pascal/std/tui/app/modals.md).
