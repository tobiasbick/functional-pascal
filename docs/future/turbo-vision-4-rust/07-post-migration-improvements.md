# Post-Migration Improvements

Status: **planned**. The core migration (Phases 0–8) is complete. This file tracks
follow-up work that makes the Turbo Vision facade usable for real applications.

This document is written to be executed by an agent with **no prior context**.
Every phase lists exact files, an existing symbol to copy from, and verifiable
done criteria. Implement **one phase per change**. Do not start a later phase
before the earlier one is merged and green.

## How to work in this repo (read first)

- Read [`AGENTS.md`](../../../AGENTS.md) and
  [`.agents/skills/fpas-change-checklist/SKILL.md`](../../../.agents/skills/fpas-change-checklist/SKILL.md).
- One concern per file; keep files under ~400–500 LOC; do not add `utils.rs`.
- Verify after every phase (PowerShell, repo root `d:\projects\functional-pascal`):

  ```text
  cargo fmt
  cargo build
  cargo test --workspace
  ```

  When `.fpas` under `tests/`, `examples/`, or `apps/` changed, also run:

  ```text
  cargo run -p fpas-cli -- test tests/
  cargo run -p fpas-cli -- fmt --check tests/ examples/ apps/
  ```

- Do not put planned behavior in `docs/pascal/`. Current docs describe only what
  exists. Plans stay in this directory.

## Verified upstream facts (crate `turbo-vision` 1.3.1)

Do not re-derive these from memory; they were checked against the crate source at
`~/.cargo/registry/src/*/turbo-vision-1.3.1/`. Re-verify only if the version
changes.

- `Dialog::execute(&mut self, app: &mut Application) -> CommandId`
  (`src/views/dialog.rs:122`) runs a dialog **modally** and returns the command
  that closed it. This is the primitive for Phase A. Mirror how
  `FileDialog::execute` is already used in
  `crates/fpas-vm/src/vm/execute/io/tui/file_dialog.rs`.
- `InputLine::new(bounds, max_length, data: Rc<RefCell<String>>)`
  (`src/views/input_line.rs:39`) and `InputLine::get_text(&self) -> String`
  (`:91`). The text lives in the shared `Rc<RefCell<String>>`, so a clone of that
  `Rc` kept host-side is readable **after** `execute` returns. This is the read-back
  mechanism for Phase A.
- `CheckBox::is_checked(&self) -> bool` (`src/views/checkbox.rs:61`) and
  `set_checked` (`:56`). CheckBox has **no** shared value cell like InputLine, so
  reading it back after `execute` needs child access on the dialog view — treat
  CheckBox/ListBox read-back as a stretch goal, not part of the Phase A minimum.
- `ListBox` has no public "selected index" getter in 1.3.1. Do not promise
  ListBox read-back until upstream support is confirmed.
- `Application::run` handles only built-in commands (`CM_QUIT = 24`,
  `CM_TILE = 29`, `CM_CASCADE = 30`, `CM_SCREENSHOT = 31`, help) and drops the
  rest; this is why `tv_run.rs` drives its own loop (see Already Landed).

## Reference recipe: add one `Std.Tui` call end to end

Every phase that adds a call touches the same 6 layers in this order. Copy an
existing call and rename. Two good models:

- **Widget/create call** → copy `Application.CreateInputLine`.
- **Modal call returning a value** → copy `Application.RunFileDialog`.

| # | Layer | File | Anchor to copy from |
|---|-------|------|---------------------|
| 1 | Symbol name const | `crates/fpas-std/src/std_units/symbols/std_symbols.rs` | `STD_TUI_APPLICATION_RUN_FILE_DIALOG` (line ~273) — add `pub const STD_TUI_APPLICATION_<NAME>: &str = std_tui!("Application.<Name>");` |
| 2 | Bytecode discriminant | `crates/fpas-bytecode/src/intrinsic/tui/variants/widgets.inc` | add `<Name> = 452,` (highest used is `CreateTextViewer = 451`; use the next free integer). Variants are stitched into `TuiIntrinsic` by `crates/fpas-bytecode/build.rs`. |
| 3 | Sema signature | `crates/fpas-sema/src/std_registry/loaded/tui/application_api.rs` | `define_func(checker, s::STD_TUI_APPLICATION_RUN_FILE_DIALOG, vec![...], returnTy)` (line ~156). Use `define_proc` for no return. New record types are registered in `crates/fpas-sema/src/std_registry/loaded/tui/mod.rs`. |
| 4 | Compiler lowering | `crates/fpas-compiler/src/compiler/std_calls/tui/application.rs` | the `s::STD_TUI_APPLICATION_RUN_FILE_DIALOG => { ... emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::RunFileDialog), location); Ok(true) }` arm (line ~156). Add a matching arm; args are compiled left to right, so pop them in reverse in the VM. |
| 5 | VM runtime | `crates/fpas-vm/src/vm/execute/io/tui/` | add a method, then dispatch it from the match in `mod.rs` (`try_exec_turbo_vision_intrinsic`, line ~52). Modal example: `file_dialog.rs`. Handle/state types live in `crates/fpas-vm/src/vm/shared.rs` (`TurboVisionObject`, `TurboVisionInputLine`, …). Record push/pop helpers live in `records.rs` and `handles.rs`. |
| 6 | Docs + tests | `docs/pascal/std/tui/app/README.md` (+ `modals.md`/`controls.md`/`types.md`), `crates/fpas-vm/src/tests/core/tui_turbo_vision_vm.rs`, `tests/tui/controls/*_test.fpas` | copy an existing table row and an existing test. |

Notes for the VM layer:

- Headless vs terminal: check `self.with_tui(|tui| tui.session.is_headless())`.
  Headless tests must not touch a real terminal — return a queued/test value
  (see `turbo_vision_run_file_dialog` and `test_file_dialog_result`).
- Main-task guard: modal/run calls must reject `self.current_task_id != 0` with a
  clear diagnostic (copy from `file_dialog.rs`).
- After adding a VM method, wire it into the match arm in `mod.rs`.

---

## Priority Order

1. Phase A — modal `ExecDialog` returning the end command (+ InputLine read-back).
2. Phase B — real multi-item menus.
3. Phase C — live widget tree during run.
4. Phase D — dual-architecture clarity (docs).
5. Phase E — command-id collision guard.
6. Phase F — live-loop testability seam.
7. Phase G — optional raw key/mouse hook.

## Already Landed

- **Live command routing (2026-07-01).**
  `crates/fpas-vm/src/vm/execute/io/tui/tv_run.rs` now steps the Turbo Vision
  event pump itself (`turbo_vision_interactive_run`) instead of calling upstream
  `Application::run`. Any event still typed as `EventType::Command` after
  `handle_event` is dispatched into the FPAS `OnCommand` callback, and an
  `Application.Quit` from that callback ends the loop.

- **Modal dialog read-back (2026-07-01).**
  `Application.ExecDialog`, `Application.InputText`, `Application.TestSetDialogResult`,
  and `DialogResult` — headless round-trip in
  `tests/tui/controls/tui_turbo_vision_exec_dialog_test.fpas`.

- **Multi-item menus (2026-07-01).**
  `Menu` / `MenuItem` replace flat `MenuBarItem`. `Application.CreateMenuBar` accepts
  `array of Menu` with multiple items and separators (`commandId = 0`).
  `Application.TestDispatchMenuCommand` drives headless menu command tests.

- **Live widget tree during run (2026-07-01).**
  `Application.Run` reconciles FPAS-side Turbo Vision mutations after each command step.
  Headless runs paint on-desktop windows and dialogs into the CRT buffer so
  `Application.QueryScreenCell` observes roots created inside `OnCommand`. Interactive runs
  mirror new windows and new dialogs onto the live Turbo Vision desktop. See `reconcile.rs`,
  `headless_paint.rs`, `tests/tui/controls/tui_turbo_vision_live_tree_test.fpas`, and
  `tests/tui/controls/tui_turbo_vision_live_dialog_test.fpas`.
  Known limits: reconciliation adds **new top-level roots** (windows, dialogs); it does not yet
  re-render property changes to already-shown views, and edits in a live (non-modal) dialog are
  not committed back to FPAS input handles — use `Application.ExecDialog` for modal read-back.

- **Dual-architecture clarity (2026-07-01).**
  Documented the Turbo Vision facade vs hosted canvas split in
  `docs/pascal/std/tui/README.md` and `docs/pascal/std/tui/app/README.md`.
  `Application.Run` selects the backend from whether any Turbo Vision handle exists.
  Mixing `Application.Configure` with widget `Create*` calls is documented as unsupported.

- **Command-id collision guard (2026-07-01).**
  FPAS command ids `24`, `29`, `30`, and `31` are offset into a private band when
  passed to Turbo Vision widgets and restored before `OnCommand` dispatch
  (`command_map.rs`). `Command.Accept` / `Cancel` / `Close` / `Quit` semantics are
  unchanged. See `tests/tui/controls/tui_turbo_vision_reserved_command_test.fpas`.

- **Live-loop testability seam (2026-07-01).**
  `interactive_loop.rs` introduces `TurboVisionInteractiveSession` so the interactive
  run loop can be driven by a scripted event source in Rust tests without a terminal.
  See `turbo_vision_scripted_interactive_loop_dispatches_command_and_quits` in
  `crates/fpas-vm/src/tests/core/tui_turbo_vision_vm.rs`.

- **Optional raw key/mouse hooks (2026-07-01).**
  `Application.OnKey` and `Application.OnMouse` register opt-in Turbo Vision fallbacks for
  keyboard and mouse events still typed after `handle_event`. Separate from hosted
  `Application.Configure` handlers. See `tv_input_events.rs` and
  `turbo_vision_scripted_interactive_loop_dispatches_unhandled_key` in
  `crates/fpas-vm/src/tests/core/tui_turbo_vision_vm.rs`.

---

## Phase A: Modal `ExecDialog` (+ InputLine read-back)

**Problem.** The facade is write-only: FPAS can build a dialog with an
`InputLine` but cannot read the user's input after it closes. Generalize the
`RunFileDialog` modal-return pattern.

**Scope (minimum).** Return the end command for any dialog, plus read back the
text of `InputLine` children. CheckBox/ListBox read-back is out of scope for this
phase (see upstream facts).

**FPAS API to add.**

```pascal
type DialogResult = record
  command: integer;   { the Command.* that closed the dialog }
end;

{ Run a dialog modally on the terminal; returns the closing command. }
function Application.ExecDialog(App: Application; Dialog: Dialog): DialogResult;

{ Read the current text of an input line (valid after ExecDialog). }
function Application.InputText(App: Application; Field: InputLine): string;
```

Rationale: `DialogResult` starts as a single field so the record can grow later
without breaking callers. `InputText` reads the retained `Rc<RefCell<String>>`.

**Step-by-step.**

1. **State.** In `crates/fpas-vm/src/vm/shared.rs`, add a retained text cell to
   `TurboVisionInputLine` (an `Rc<RefCell<String>>`). When
   `turbo_vision_create_input_line` (in `controls.rs`) builds the handle, create
   the `Rc` and store it. `TurboVisionObject` is `Clone`; wrap the cell so cloning
   shares it (`Rc` already shares).
2. **Build reuse.** In `tv_run.rs`, where `add_dialog_child` builds an
   `InputLine`, pass the stored `Rc` clone into `InputLine::new(..., rc.clone())`
   instead of `Rc::new(RefCell::new(text))`. This makes edits observable
   host-side.
3. **ExecDialog VM method.** Add `crates/fpas-vm/src/vm/execute/io/tui/exec_dialog.rs`
   (new module; register it in `mod.rs`). Copy the terminal/headless/main-task
   structure from `file_dialog.rs`. Build a single `Dialog` view (from the given
   dialog handle's snapshot + children, reuse the `add_dialog_child` helper), call
   `dialog.execute(&mut app)`, and push a `DialogResult` record with the returned
   command. Headless path: return a test-provided command (add
   `test_dialog_result` to the `turbo_vision` state, plus a
   `TestSetDialogResult` call following `TestSetFileDialogResult`).
4. **InputText VM method.** Pop an `InputLine` handle, read its `Rc` borrow, push
   the string. Unknown handle → `unknown_handle_error` (see `tv_geometry.rs`).
5. **Layers 1–4 + 6** of the reference recipe for both new calls
   (`Application.ExecDialog`, `Application.InputText`) and the `DialogResult`
   record type (register its fields in `loaded/tui/mod.rs` like `Size`).
6. **Docs.** Update `docs/pascal/std/tui/app/modals.md` and the call table in
   `docs/pascal/std/tui/app/README.md`.

**Tests.**

- FPAS headless (`tests/tui/controls/tui_turbo_vision_exec_dialog_test.fpas`):
  open for test, create a dialog + input line, seed input text via the test event
  queue, set the test dialog result to `Command.Accept`, `ExecDialog`, assert the
  returned command and that `Application.InputText` returns the seeded text.
- Rust (`crates/fpas-vm/src/tests/core/tui_turbo_vision_vm.rs`): assert the input
  line `Rc` read-back invariant (edit via the cell, read via the handle).

**Definition of done.** A headless test round-trips an `InputLine` value back to
FPAS and observes the closing command.

**Go/no-go.** `ExecDialog` returns the closing command; `InputText` returns the
edited value in a headless test.

---

## Phase B: Real multi-item menus

**Problem.** `turbo_vision_create_menu_bar`
(`crates/fpas-vm/src/vm/execute/io/tui/navigation.rs`, ~line 30) and
`build_menu_bar` (`tv_run.rs`) put exactly one `MenuItem` per top-level menu, so a
"File" menu holds a single command. Real menus need multiple items and
separators.

**FPAS API to add (replaces flat `MenuBarItem`).**

```pascal
type MenuItem = record
  text: string;        { '~O~pen' }
  commandId: integer;  { use 0 for a separator }
end;

type Menu = record
  title: string;       { '~F~ile' }
  items: array of MenuItem;
end;

function Application.CreateMenuBar(App: Application; Bounds: Rect;
                                   Menus: array of Menu): MenuBar;
```

**Files.** Record types in `loaded/tui/mod.rs` (replace the `MenuBarItem`
registration, ~line 71). Shared model in `shared.rs`
(`TurboVisionMenuBarItem` → nested `Menu`/`MenuItem`). Build in `navigation.rs`
and `tv_run.rs::build_menu_bar`: use `Menu::from_items(vec![...])` with **all**
items, mapping `commandId == 0` to a separator (check
`turbo-vision-1.3.1/src/core/menu_data.rs` for the separator constructor).
Migrate `apps/ide/src/menu.fpas`, `examples/pascal/tui/*.fpas`, and the TUI menu
tests.

**Tests.** Headless dispatch of a command from a **non-first** menu item; assert
`OnCommand` receives the right id. Keep a single-item menu working.

**Definition of done.** A menu with ≥2 items and a separator dispatches the
correct command for a non-first entry.

---

## Phase C: Live widget tree during run

**Status:** landed (2026-07-01). See **Already Landed** above.

**Problem.** `build_turbo_vision_application` (`tv_run.rs`) snapshots FPAS state
once before the loop. Commands are now live, but widgets created or mutated inside
`OnCommand`, and `Application.RequestRedraw`, have no visible effect during a run.

**Approach.** Replace the one-shot snapshot with a live handle↔view mapping owned
for the duration of `turbo_vision_interactive_run`, applying FPAS-side mutations
between event steps. Depends on Phase A's retained-handle work; do A and B first.

**Files.** `tv_run.rs` (loop owns the app and reconciles a pending
create/mutate queue each turn), `shared.rs` (pending-mutation queue on the
`turbo_vision` state), `controls.rs`/`windows.rs`/`dialogs.rs` (apply mutations to
live views).

**Tests.** Headless: an `OnCommand` handler adds a window; a later screen query
observes it.

**Definition of done.** A widget created inside `OnCommand` is observable in a
headless screen query after the next step.

---

## Phase D: Dual-architecture clarity (docs)

**Status:** landed (2026-07-01). See **Already Landed** above.

**Problem.** `Std.Tui` exposes two paradigms: the hosted immediate-mode loop
(`ApplicationHandlers` with `OnPaint`/`OnKeyPressed` drawing through
`Std.Console`, used by `examples/math/mandelbrot` and
`examples/pascal/tui/minimal_application.fpas`) and the Turbo Vision retained
facade. `Application.Run` branches on whether Turbo Vision handles exist.

**Action.** Document the boundary clearly in `docs/pascal/std/tui/README.md` and
the app hub: when to use the hosted canvas vs the Turbo Vision widgets, and that
mixing them in one app is unsupported. Recommendation: keep both, document
sharply. No code change unless the team decides to drop the hosted loop (larger,
separate decision).

**Definition of done.** A reader can decide within one page which API to use.

---

## Phase E: Command-id collision guard

**Status:** landed (2026-07-01). See **Already Landed** above.

**Problem.** FPAS command ids share the integer space with upstream built-ins
(`CM_QUIT = 24`, `CM_TILE = 29`, `CM_CASCADE = 30`, `CM_SCREENSHOT = 31`). A user
id equal to one of those triggers unintended Turbo Vision behavior.

**Approach.** Offset user command ids into a reserved high band before handing
them to Turbo Vision (in `controls.rs`/`navigation.rs` at create time) and
translate back before FPAS dispatch (in `callbacks.rs`), keeping
`Command.Accept/Cancel/Close/Quit` semantics. Alternative: validate at create time
with a diagnostic listing the reserved ids.

**Tests.** Rust: a user id of 24 reaches `OnCommand` unchanged and does not quit
the app.

**Definition of done.** A user command with id 24 reaches `OnCommand` and does not
quit.

---

## Phase F: Live-loop testability seam

**Status:** landed (2026-07-01). See **Already Landed** above.

**Problem.** The interactive loop needs a real `Terminal` (`Application::new`
calls `Terminal::init`), and upstream's `MockTerminal` is not pluggable into
`Application`, so the "unhandled command → FPAS" invariant is proven only by
inspection plus the shared dispatch unit test.

**Approach.** Extract the loop's event source in `tv_run.rs` behind a small
internal trait (one method: next event) so a fake source can drive
`turbo_vision_interactive_run` in a Rust test without a terminal. Keep the
production path on the real `Application`.

**Files.** `tv_run.rs` (introduce the seam), new test in
`crates/fpas-vm/src/tests/core/`.

**Definition of done.** A headless Rust test drives the interactive loop with a
scripted button command and asserts the FPAS callback fired and quit ended the
loop.

---

## Phase G (optional): Raw key/mouse hook in the Turbo Vision path

**Status: landed (2026-07-01).**

Only if a concrete use case appears. Today the Turbo Vision path routes commands
to FPAS but not raw keyboard/mouse events. If needed, add an opt-in
`OnKey`/`OnMouse` hook in `tv_run.rs` that fires for events left unhandled by the
view tree. Otherwise leave the facade command-only.

**Landed as:** `Application.OnKey`, `Application.OnMouse`, `tv_input_events.rs`,
dispatch from `interactive_loop.rs` after `handle_event`.
