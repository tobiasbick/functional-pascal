# Rust layout

Target module layout after try-2. Follow [AGENTS.md](../../AGENTS.md): one concern per file, subdirectories over flat growth.

## VM bridge (`crates/fpas-vm/src/vm/execute/io/tui/`)

### Current `try2/` tree (branch tip, 2026-07-10)

Phase 7 is removing the remaining try-1 bridge modules. Try-2 owns all public view construction, run, modal, and callback routes.

```text
crates/fpas-vm/src/vm/execute/io/tui/try2/
  mod.rs
  session.rs       — Try2Session on Worker (registry, roots, detached widgets, desktop windows)
  registry.rs      — ViewRegistry + ViewKind
  geometry.rs      — FPAS Rect ↔ turbo_vision Rect
  records.rs       — opaque handle record helpers
  events.rs        — OnCommand dispatch (no command_map offset)
  run.rs           — Application.Run (headless + live)
  headless.rs      — headless ExecView ensure + export
  modals.rs        — Application.ExecView
  app.rs           — live Application::new (no try-1 populate)
  application_records.rs — `Application` and `Size` VM value constructors
  chrome.rs        — MenuBar/StatusLine.New + SetMenuBar/SetStatusLine sync
  chrome_input.rs  — MenuBar/StatusLine record decoding
  commands.rs      — callback registration, Quit, and headless menu dispatch (pending split)
  message_box.rs   — Application.MessageBox try-2 route
  file_dialog.rs   — Application.RunFileDialog try-2 route (live upstream, headless Try2Session queue adapter)
  view_lookup.rs   — lookup attached child views for live mutation/read-back
  intrinsics.rs    — try-2 VM dispatch
  testing.rs       — Test.Click, Test.DispatchMenu, TestClickMouse try-2 paths
  views/
    mod.rs
    dialog.rs      — Dialog.NewModal
    button.rs      — Button.New
    static_text.rs — StaticText.New
    attach.rs      — Dialog.Add / Window.Add child dispatch
    window.rs      — Window.New, Window.Add
    desktop.rs     — Desktop.Add
    input_line.rs   — InputLine.New/Text/SetText
    list_box.rs     — ListBox.New/Selection/SetItems
    check_box.rs    — CheckBox.New/Checked/SetChecked
    radio_button.rs — RadioButton.New/Selected/SetSelected
    memo.rs         — Memo.New/SetText
    text_viewer.rs  — TextViewer.New/SetText
```

`HeadlessTvApp` now lives in `try2/headless_draw.rs` for headless paint, modal execution, and run-loop input.
`CheckBox`, `RadioButton`, and `Outline` still use small `Bridged*` view types because their pinned upstream types do not provide `View::as_any_mut`; the wrappers preserve read-back after live input. This remains a Phase-7 cleanup target.

### Historical target tree (superseded)

This was the pre-Phase-7 consolidation proposal. It is retained as historical planning context; the authoritative current tree is the `Current try2/ tree` above and the public bridge map in [`docs/pascal/std/tui/app/vm-bridge.md`](../pascal/std/tui/app/vm-bridge.md). Phase 7 now only awaits removal of the three upstream-dependent `bridged_*` adapters.

```text
crates/fpas-vm/src/vm/execute/io/tui/
  mod.rs                 — module declarations, intrinsic dispatch table
  session.rs             — Open/Close/Size, live app ensure/drop
  registry.rs            — FpasViewId ↔ ViewId map, handle validation
  run.rs                 — Application.Run loop, Quit
  events.rs              — Command/Key/Mouse callback dispatch
  modals.rs              — ExecView, MessageBox, RunFileDialog
  chrome.rs              — MenuBar/StatusLine build + set on app
  geometry.rs            — Rect conversion
  headless.rs            — OpenForTest, terminal backend, event injection
  testing.rs             — Test.Click, Test.InjectEvent, screen draw export
  views/
    mod.rs
    dialog.rs            — Dialog.New, NewModal, Add overloads, SetTitle
    window.rs            — Window.New, Add, SetTitle
    desktop.rs           — Desktop.Add
    button.rs
    static_text.rs
    input_line.rs
    list_box.rs
    check_box.rs
    radio_button.rs
    memo.rs
    text_viewer.rs
    outline.rs           — phase 2
```

**Estimated size:** ~12–15 modules, ~1.5–2.5k LOC total.

### `mod.rs` dispatch

Keep a single `match` on `TuiIntrinsic` delegating to view modules — same pattern as today, fewer variants that do real work.

### `registry.rs` (new)

```rust
//! FpasViewId assignment and ViewId lookup for live tree operations.
```

Responsibilities:

- Allocate monotonic `u32` handles
- Store `ViewId` + `ViewKind` + optional parent `FpasViewId`
- Validate handle on every intrinsic
- On `Close`, clear all entries

No bounds/text/children in registry.

### `session.rs` (from `session_app.rs` + `application.rs`)

Merge:

- `turbo_vision_ensure_live_app`
- `turbo_vision_with_live_app`
- `Application.Open` / `Close` / `Size`
- Drop snapshot sync (`turbo_vision_sync_chrome_from_fpas` → only read chrome **from** FPAS records at set time, not from snapshot enum)

### `run.rs` (from `tv_run.rs` + `tui_run.rs`)

- Interactive: call upstream `run()` or equivalent loop with FPAS callback hooks
- Remove full desktop rebuild path
- Remove `turbo_vision_begin_run` reconcile bookkeeping

### `headless.rs` (simplify)

Merge:

- `tv_headless_backend.rs` (keep if still needed)
- Parts of the former root headless renderer — only CRT export after `draw`
- Delete separate `HeadlessTvApp` widget tree builder

### Worker changes (`crates/fpas-vm/src/vm/worker.rs`)

```rust
// Keep
live_turbo_vision_app: Option<turbo_vision::app::Application>,

// Add
tv_view_registry: ViewRegistry,

// TuiState: strip turbo_vision: TurboVisionState
```

### Shared state (`crates/fpas-vm/src/vm/shared/tui.rs`)

Reduce to:

```rust
pub(crate) struct TuiState {
    pub session: TuiSession,
    pub on_command: Option<Value>,
    pub on_key: Option<Value>,
    pub on_mouse: Option<Value>,
    pub quit_requested: bool,
}
```

Delete `TurboVisionState`, `TurboVisionObject`, and all snapshot structs.

### Delete entirely (try-1 modules)

| File | Reason |
| --- | --- |
| `reconcile.rs` | No dual-state reconcile |
| `live_patch.rs` | Direct live mutation |
| `bridged_*.rs` (3 files) | No adapter views |
| `command_map.rs` | No offset band |
| `control_create.rs` | Replaced by `views/*` |
| `tv_views.rs` | Replaced by `views/*` |
| `controls.rs` | Split into `views/*` |
| `dialogs.rs`, `windows.rs` | Merged into `views/dialog.rs`, `views/window.rs` |
| `navigation.rs` | → `chrome.rs` |
| `menu_build.rs` | → `chrome.rs` or `views/menu_bar.rs` |
| `chrome_layout.rs` | → `chrome.rs` |
| `handle_records.rs`, `handles.rs`, `records.rs` | → `registry.rs` + `geometry.rs` |
| `interactive_loop.rs` | → `run.rs` + tests |
| `commands.rs` | → `run.rs` + `testing.rs` |
| `test_mouse.rs` | → `testing.rs` |
| `outline_nodes.rs`, `outline_read.rs` | → `views/outline.rs` (phase 2) |
| `callbacks.rs` | → `events.rs` |
| `tv_input_events.rs` | → `events.rs` |
| `exec_dialog.rs` | → `modals.rs` |
| `file_dialog.rs` | → `modals.rs` |
| `msgbox.rs` | → `modals.rs` |
| `lifecycle.rs` | → `session.rs` + `events.rs` |
| `testing.rs` | Rewrite in place |
| former root `headless_tv_draw.rs` | → `try2/headless_draw.rs` |
| `application.rs` | → `session.rs` |

Also remove `turbo_vision_*_cell.rs` bridge cells if read-back uses live view state directly.

## `fpas-std` (`crates/fpas-std/src/tui/`)

```text
crates/fpas-std/src/tui/
  mod.rs
  cm_constants.rs      — selected CM_* constants generated by `fpas-std/build.rs`
  command.rs           — retained internal command dispatch
  session/             — keep session/redraw for CRT back buffer
  event.rs             — keep ExitReason if needed
```

Delete or repurpose `command_ids.rs` try-1 `COMMAND_*` values.

## Sema (`crates/fpas-sema/src/std_registry/loaded/tui/`)

```text
loaded/tui/
  mod.rs
  types.rs             — Application, Rect, opaque view types
  command_api.rs       — CM_* constants
  application.rs       — New, Run, Quit, ExecView, Configure (optional)
  handlers.rs          — OnKey/OnMouse signatures
  views/
    mod.rs
    dialog.rs
    window.rs
    button.rs
    …
```

Delete monolithic `application_api.rs` after split.

## Compiler (`crates/fpas-compiler/src/compiler/std_calls/tui/`)

Mirror sema layout:

```text
std_calls/tui/
  mod.rs
  application.rs
  views/
    dialog.rs
    button.rs
    …
```

## Bytecode (`crates/fpas-bytecode`)

- Add new `TuiIntrinsic` variants for try-2 API.
- Remove old `TuiCreate*` / `TuiAddChild` variants in same change (no compat).
- Update intrinsic display names in diagnostics.

## Docs (post-implementation)

Replace `docs/pascal/std/tui/` structure:

```text
docs/pascal/std/tui/
  README.md
  session.md
  commands.md          — CM_* reference
  application.md       — Run, Quit, ExecView, modals
  views/
    README.md
    dialog.md
    window.md
    button.md
    …
  testing.md
  vm-bridge.md         — short contributor map (15 modules, not 40)
```

## Skill update

After implementation, rewrite [`.agents/skills/turbo-vision-4-rust/SKILL.md`](../../.agents/skills/turbo-vision-4-rust/SKILL.md) architecture section:

- Remove “do not mirror one-to-one”
- Document Rust-owned tree + record method API
- Update expected file layout table

## Dependency

No change to workspace pin unless bumping for test-util APIs:

```toml
turbo-vision = { git = "…", tag = "v2.0.0", features = ["test-util"] }
```
