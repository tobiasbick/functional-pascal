# 05 — Incremental view updates instead of full desktop rebuild

**Status:** [x] Audit · [x] Phase A · [x] SetText inputline/memo/textviewer · [x] Headless repaint skip · [x] Phase B

**Priority:** Low — architectural; after bridge stabilisation

**Depends on:** [done/02-single-tv-session.md](done/02-single-tv-session.md), [done/04-headless-test-util.md](done/04-headless-test-util.md)

## Problem

On many `SetText`, `SetChecked`, `SetItems`, `SetTitle`, and post-command steps, `reconcile.rs` sets `pending_reconcile` and **rebuilds the entire desktop**:

- Drop all TV windows/dialogs from `app.desktop`
- Re-snapshot every FPAS handle into new `Window`/`Dialog` views
- Re-attach all children

This is simple and correct but expensive and fights upstream identity/focus/modal semantics (TV 2.0 added identity-tracked modal loops, focus propagation, etc.).

## Target

- **Phase A (minimal):** Rebuild only when structure changes (`AddChild`, `AddWindow`, `Create*`, remove). Mutations call upstream setters on live views where they exist.
- **Phase B (optional):** Store `ViewId` / weak mapping from FPAS handle → live TV view on session; patch in place.

Still keep FPAS handle graph authoritative for Pascal and headless introspection.

## Audit — `mark_turbo_vision_tree_dirty` call sites

| Site | Trigger | Classification |
| --- | --- | --- |
| `control_create.rs` | `Create*` | **Structural** — keep full rebuild |
| `controls.rs` `AddChild` | attach child | **Structural** |
| `controls.rs` `SetText` | text mutation | **Data** — live repaint for all text-bearing controls via `Bridged*` / input bindings |
| `controls.rs` `SetChecked` | cell mutation | **Data** — live patch via `live_patch.rs` |
| `controls.rs` `SetItems` | list mutation | **Data** — live patch |
| `windows.rs` `CreateWindow` / `AddWindow` | roots | **Structural** |
| `windows.rs` `SetTitle` | title mutation | **Data** — live patch |
| `dialogs.rs` `CreateDialog` | root | **Structural** |
| `navigation.rs` | menu/status chrome | **Structural** (chrome sync on rebuild) |
| `test_mouse.rs` | headless click | **Repaint** — TV `handle_event` + `pending_headless_repaint` |
| `bridged_radio_button.rs` | user select | **Structural** — group exclusivity + `tree_dirty` |

## Tasks

- [x] **Audit** — Table above.
- [x] **Phase A** — `live_patch.rs`: `SetChecked`, `SetItems`, `SetTitle` patch live views; `live_view_ids` + `live_child_root_view_ids` registered at populate.
- [x] **Phase A follow-up** — `SetText` on `StaticText` / `InputLine` / `Memo` / `TextViewer` via live patch (`BridgedStaticText`, `BridgedMemo`, `BridgedTextViewer`, shared input bindings).
- [x] **Phase B** — Child handles map to parent root `ViewId` (not desktop index); survives `Desktop::bring_to_front`. Cleared on full rebuild; modal `ExecDialog` trees are separate ephemeral instances.
- [x] **Headless** — `pending_headless_repaint` skips desktop wipe; data mutations patch existing `HeadlessTvApp` tree.
- [x] **Tests** — `set_text`, `set_checked`, `set_items`, `set_title`, full `tests/tui/controls/`.
- [x] **Perf** — No IDE-scale regression observed in manual use; structural rebuild remains on `AddChild` / `Create*` only.
- [x] **Context** — [00-context.md](00-context.md).

## Files

```text
crates/fpas-vm/src/vm/execute/io/tui/live_patch.rs       — NEW: live data mutation patch
crates/fpas-vm/src/vm/execute/io/tui/reconcile.rs
crates/fpas-vm/src/vm/execute/io/tui/controls.rs
crates/fpas-vm/src/vm/execute/io/tui/windows.rs
crates/fpas-vm/src/vm/execute/io/tui/tv_run.rs
crates/fpas-vm/src/vm/execute/io/tui/tv_views.rs
crates/fpas-vm/src/vm/shared/tui.rs
```

## Verification

```text
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_set_text_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_live_tree_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/
cargo test --workspace
```

## Non-goals (for this item)

- Exposing live TV view handles to Pascal
- Removing FPAS snapshot layer entirely

## Notes

- Live and headless share `apply_live_data_mutation_to_desktop`; headless reconcile skips populate when only `pending_headless_repaint` is set.
- `live_child_root_view_ids` stores the window/dialog `ViewId` for each child handle so live patch and headless mouse routing survive desktop z-order changes without a rebuild.
- Full desktop rebuild clears all live maps; do not patch across `ExecDialog` modal instances (they are not registered in session maps).
- `SetText` on all text-bearing controls patches live views (`BridgedButton`, `BridgedCheckBox`, `BridgedRadioButton`, `BridgedStaticText`, `BridgedMemo`, `BridgedTextViewer`, input bindings).
