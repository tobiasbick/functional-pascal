# 05 — Incremental view updates instead of full desktop rebuild

**Status:** [ ] Not started · [ ] In progress · [ ] Done

**Priority:** Low — architectural; do after 01 and preferably 03

**Depends on:** [done/02-single-tv-session.md](done/02-single-tv-session.md), [03-headless-test-util.md](03-headless-test-util.md) (recommended)

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

## Tasks

- [ ] **Audit** — List all calls to `mark_turbo_vision_tree_dirty` / `pending_reconcile`; classify structural vs data mutation.
- [ ] **Upstream API** — For each control, confirm TV 2.0 setters (`set_text`, `set_checked`, list `set_items`, …) match bridge needs.
- [ ] **Phase A** — Narrow reconcile triggers; implement live updates in `controls.rs` for `SetText`, `SetChecked`, `SetItems`, `SetTitle` without full rebuild (partially exists — verify completeness).
- [ ] **Phase B** — Design handle→view map on session; document re-entrancy and modal cases.
- [ ] **Remove** — Full rebuild on hot paths where incremental path proven.
- [ ] **Tests** — `tui_turbo_vision_set_text_test.fpas`, `set_checked`, `set_items`, `set_title`, live tree tests.
- [ ] **Perf** — Optional benchmark or manual note if IDE-scale trees feel sluggish before/after.
- [ ] **Context** — Update [00-context.md](00-context.md).

## Files (expected touch)

```text
crates/fpas-vm/src/vm/execute/io/tui/reconcile.rs
crates/fpas-vm/src/vm/execute/io/tui/controls.rs
crates/fpas-vm/src/vm/execute/io/tui/tv_views.rs
crates/fpas-vm/src/vm/shared/tui.rs
```

## Verification

```text
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_set_text_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_live_tree_test.fpas
cargo test --workspace
```

## Non-goals (for this item)

- Exposing live TV view handles to Pascal
- Removing FPAS snapshot layer entirely

## Notes

- TV 2.0 `Event.info` field may matter for scroll/history commands — reconcile with [04-command-map-sync.md](04-command-map-sync.md) if bridging new broadcasts.
