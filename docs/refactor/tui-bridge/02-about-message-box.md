# 02 — About and simple dialogs via upstream `message_box`

**Status:** [ ] Not started · [x] In progress · [ ] Done

**Priority:** Medium (quick win; session model is in place)

**Depends on:** [done/02-single-tv-session.md](done/02-single-tv-session.md) — completed. Live About during `Run` uses `Worker.live_turbo_vision_app` via `turbo_vision_with_live_app`.

**Blocks:** [07-pascal-message-box-api.md](07-pascal-message-box-api.md) (optional public API)

## Problem

`apps/ide/src/dialog/about.fpas` manually composes `CreateDialog`, `CreateTextViewer`, `CreateButton`, and `ExecDialog` (~57 lines) to mimic Turbo Vision’s About box. Upstream already provides:

- `turbo_vision::helpers::msgbox::{message_box, message_box_ok, message_box_rect, …}`
- `MF_ABOUT | MF_OK_BUTTON` title and layout (see upstream `src/helpers/msgbox.rs` at pinned tag `v2.0.0`)

We duplicate layout, sizing, and Borland conventions in FPAS.

## Current IDE layout (today)

| Piece | FPAS (`about.fpas`) | Upstream `message_box` |
| --- | --- | --- |
| Title | `AboutTitle()` → `'About ℹ️'` | `MF_ABOUT` → `"About ℹ️"` |
| Body | `CreateTextViewer` + `AboutMessage()` | `StaticText` over split `\n` lines |
| OK button | `CreateButton` … `Command.Accept` | `MF_OK_BUTTON` → `CM_OK` (10) |
| Bounds | `AboutDialogBounds` from `Application.Size` | Auto-sized from message + terminal size |
| Modal run | `Application.ExecDialog` | `Dialog::execute` inside helper |

Headless IDE tests (`dialog_test.fpas`, `about_menu_test.fpas`) queue `Command.Accept` via `TestSetDialogResult`; keep that path until [03-headless-test-util.md](03-headless-test-util.md) or a dedicated msgbox test stub.

## Upstream API (turbo-vision 2.0.0)

```rust
use turbo_vision::helpers::msgbox::{message_box, MF_ABOUT, MF_OK_BUTTON};
use turbo_vision::core::command::CM_OK; // 10 — same as Borland cmOK / FPAS Command.Accept

let cmd = message_box(app, "FPAS IDE\n\nFunctional Pascal IDE\n\nBuilt with Turbo Vision", MF_ABOUT | MF_OK_BUTTON);
// cmd == CM_OK when user presses OK
```

Type flags (lower 4 bits): `MF_WARNING`, `MF_ERROR`, `MF_INFORMATION`, `MF_CONFIRMATION`, `MF_ABOUT`.

Button flags: `MF_YES_BUTTON`, `MF_NO_BUTTON`, `MF_OK_BUTTON`, `MF_CANCEL_BUTTON`; combos `MF_OK_CANCEL`, `MF_YES_NO_CANCEL`.

Return value is upstream `CommandId` (`CM_OK`, `CM_CANCEL`, …). Map through `command_map.rs` / `turbo_vision_command_to_fpas` before any FPAS-visible result.

## Target

- IDE About (and similar one-shot messages) call upstream `message_box` from Rust, not hand-built FPAS widget trees.
- Same Borland look as native TV examples (`menu_status.rs`, `desktop_logo.rs`).
- Pascal IDE may call a thin internal hook or a future public API; avoid large FPAS layout code for standard message boxes.

## Bridge helper sketch

```text
crates/fpas-vm/src/vm/execute/io/tui/msgbox.rs
  turbo_vision_show_message_box(worker, line, message: &str, options: u16) -> Result<CommandId, VmError>
    → turbo_vision_with_live_app(line, |app| Ok(message_box(app, message, options)))
    → turbo_vision_command_to_fpas(cmd) for FPAS-facing ids
```

Phase 1 (this item): private `Worker` helper used from IDE — **not** a public `Application.MessageBox` until [07](07-pascal-message-box-api.md) if needed.

## IDE command flow

```text
Help menu item (commandId 100 = CM_ABOUT)
  → Turbo Vision menu dispatch
  → OnCommand(App, 100)
  → Ide.Dialog.HandleCommand → ShowAbout(App)
  → [today] ExecDialog(custom dialog) → DialogResult.command = Command.Accept (10)
  → [target] message_box(..., MF_ABOUT | MF_OK_BUTTON) → CM_OK (10)
```

Menu command `100` stays in FPAS (`CmdHelpAbout`). Only the modal implementation changes.

## Implementation stack (phase 1 — recommended)

Add an **`Application.MessageBox`** stdlib call for IDE use only. Register it in sema/compiler/bytecode/VM, but do **not** add a `docs/pascal/` page until [07](07-pascal-message-box-api.md) (vm-bridge in-progress note only).

| Layer | File | Change |
| --- | --- | --- |
| VM helper | `msgbox.rs` | `turbo_vision_show_message_box(worker, line, message, options) -> i64` |
| VM dispatch | `mod.rs`, new `message_box.rs` or `exec_dialog.rs` neighbor | `TuiIntrinsic::MessageBox` handler |
| Bytecode | `fpas-bytecode/src/intrinsic/tui/` | `TuiIntrinsic::MessageBox` variant |
| Compiler | `std_calls/tui/application.rs` | Lower `Application.MessageBox(App, Message, Options)` |
| Sema | `fpas-sema/src/std_registry/loaded/tui/` | Typecheck `(Application, string, integer) -> integer` |
| Std symbols | `fpas-std` | `STD_TUI_APPLICATION_MESSAGE_BOX` constant |
| IDE | `about.fpas` | `ShowAbout` → `Application.MessageBox(App, AboutMessage(), MF_ABOUT \| MF_OK_BUTTON)` with local flag constants or integer literal `0x404` |
| Headless | `msgbox.rs` | If `session.is_headless()`, return `test_dialog_result` (same queue as `TestSetDialogResult`) so existing IDE tests need no change |

**Alternative rejected for phase 1:** Rust-only helper with no Pascal entry — IDE `ShowAbout` is Pascal and must call a registered symbol.

**Headless interim:** Reuse `Application.TestSetDialogResult` until [03](03-headless-test-util.md); no separate `TestSetMessageBoxResult` unless tests need it.

## Headless test expectations

| Test | What it checks |
| --- | --- |
| `apps/ide/tests/dialog/dialog_test.fpas` | `HandleCommand(App, CmdHelpAbout)` with `TestSetDialogResult(Command.Accept)` |
| `apps/ide/tests/shell/about_menu_test.fpas` | `TestDispatchMenuCommand` → Help/About → `LastCommandId = CmdHelpAbout` |

After `message_box`, interactive path calls upstream directly; headless may still stub via `TestSetDialogResult` until [03](03-headless-test-util.md).

## Tasks

- [ ] **Spike** — `msgbox.rs`: call `message_box` via `turbo_vision_with_live_app`; headless branch returns queued `test_dialog_result`.
- [ ] **Bridge helper** — `turbo_vision_show_message_box` + `TuiIntrinsic::MessageBox` end-to-end (sema → compiler → VM).
- [ ] **IDE** — `ShowAbout` calls `Application.MessageBox`; remove `CreateDialog` / `AddChild` / `ExecDialog` layout from `about.fpas`.
- [ ] **Remove dead FPAS layout** — Delete `AboutDialogBounds`, `Bounds` helper if unused; keep `AboutMessage()` / title strings in Pascal.
- [ ] **Tests** — Update `apps/ide/tests/dialog/dialog_test.fpas` and `about_menu_test.fpas`; headless path may still use `TestSetDialogResult` until [03-headless-test-util.md](03-headless-test-util.md).
- [x] **Docs (plan)** — This file; [modals.md](../../pascal/std/tui/app/modals.md), [handlers.md](../../pascal/std/tui/app/handlers.md), [types.md](../../pascal/std/tui/app/types.md), [testing.md](../../pascal/std/tui/app/testing.md), [vm-bridge.md](../../pascal/std/tui/app/vm-bridge.md) (in-progress note); agent skill + api_reference example.
- [ ] **Docs (implement)** — Add `msgbox.rs` row to `vm-bridge.md` after code exists; public Pascal API waits for [07](07-pascal-message-box-api.md).

## Files (expected touch)

```text
crates/fpas-vm/src/vm/execute/io/tui/msgbox.rs
crates/fpas-vm/src/vm/execute/io/tui/mod.rs
crates/fpas-bytecode/src/intrinsic/tui/
crates/fpas-compiler/src/compiler/std_calls/tui/application.rs
crates/fpas-sema/src/std_registry/loaded/tui/
crates/fpas-std/src/std_symbols.rs (or tui symbols module)
apps/ide/src/dialog/about.fpas
apps/ide/tests/…
docs/pascal/std/tui/app/vm-bridge.md
```

## Verification

```text
cargo test --workspace
cargo run -q -p fpas-cli -- test apps/ide/tests/
cargo run -q -p fpas-cli -- apps/ide/ide.fpasprj   # manual: Help → About
```

## Notes

- Do not reimplement `message_box` sizing logic in FPAS.
- `input_box` is a separate follow-up if needed (upstream `helpers::msgbox::input_box`).
- Custom dialogs with read-back still use `CreateDialog` + `ExecDialog` — see [modals.md](../../pascal/std/tui/app/modals.md).
- `CM_OK` from upstream OK button equals `Command.Accept` (10) after mapping; no new Pascal constant required.
