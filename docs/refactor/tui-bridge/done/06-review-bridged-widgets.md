# 06 — Re-evaluate `Bridged*` view wrappers after TV 2.0

**Status:** [x] Matrix · [x] CheckBox shrink · [x] RadioButton shrink · [x] ListBox review · [x] Headless mouse follow-up · [x] Delete `*_mouse.rs` · [x] Tests · [x] Context

**Priority:** Medium — after [done/04-headless-test-util.md](done/04-headless-test-util.md)

**Depends on:** turbo-vision 2.0 behavior (radio groups, mouse on clusters, disabled commands)

## Problem

FPAS wraps upstream widgets to mirror state into VM cells and to patch input gaps:

| Module | Purpose |
| --- | --- |
| `bridged_check_box.rs` | `TurboVisionBoolCell` sync after upstream events |
| `bridged_radio_button.rs` | Group exclusivity in FPAS cells + `tree_dirty` |
| `bridged_list_box.rs` | Selection cell sync |
| `test_mouse.rs` | Headless coordinate hit-test maps FPAS logical coords to live TV view bounds before dispatch |

Turbo Vision 2.0 `Cluster::handle_cluster_event` handles left-click toggle/select inside bounds (not only Space when focused). The former `check_box_mouse.rs` / `radio_button_mouse.rs` modules duplicated that behavior.

Standard message boxes ([done/03-about-message-box.md](done/03-about-message-box.md)) bypass `Bridged*` wrappers entirely — upstream `helpers::msgbox` constructs stock views inside `message_box`.

## Target

For each wrapper, document **keep**, **shrink**, or **delete**:

- If upstream handles behavior, use stock TV types where possible and read state after events.
- If FPAS still needs cells for `ExecDialog` read-back or headless, keep minimal glue only.

## Tasks

- [x] **Matrix** — Table below.
- [x] **CheckBox** — `BridgedCheckBox` delegates mouse to upstream `CheckBox`; wrapper only syncs `TurboVisionBoolCell`.
- [x] **RadioButton** — Same for mouse; wrapper keeps FPAS group-cell exclusivity (radios are dialog siblings, not upstream `Group` children).
- [x] **ListBox** — Window chrome uses stock `ListBox`; dialogs/`ExecDialog` keep `BridgedListBox` for `ListSelection` read-back.
- [x] **Mouse** — Headless `TestClickMouse` routes through `HeadlessTvEventInbox` + desktop `handle_event` ([done/04-headless-test-util.md](done/04-headless-test-util.md) follow-up).
- [x] **Delete** — Removed `check_box_mouse.rs`, `radio_button_mouse.rs`.
- [x] **Tests** — Rust unit tests on `Bridged*`; FPAS control tests below.
- [x] **Context** — [00-context.md](00-context.md) bridge table updated.

## Verification

```text
cargo test -p fpas-vm bridged
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_check_box_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_radio_button_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_check_box_mouse_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_radio_button_mouse_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/
```

## Decision matrix

| Widget | FPAS need | TV 2.0 | Current bridge | Decision |
| --- | --- | --- | --- | --- |
| CheckBox | `Checked`, `SetChecked`, `ExecDialog` read-back | Cluster mouse + Space | `bridged_check_box.rs` | **Keep shrink** — cell sync only; upstream handles click |
| RadioButton | `Selected`, cross-handle `GroupId` exclusivity | Cluster mouse + `select()`; group broadcast needs `Group` parent | `bridged_radio_button.rs` | **Keep shrink** — FPAS group cells + `tree_dirty`; upstream handles click |
| ListBox | `ListSelection`, `OnCommand` on Enter | Full listbox | Stock in windows; `bridged_list_box.rs` in dialogs | **Keep** — selection cell required for read-back |

## Notes

- Do not remove cells if `ExecDialog` read-back still depends on them without an upstream read API.
- Headless `TestClickMouse` routes through `HeadlessTvEventInbox` and upstream cluster mouse handling; FPAS cells sync via `Bridged*` wrappers.
- `live_tree_test` title-bar glyph vs `LIVE` label visibility remains a separate follow-up.
