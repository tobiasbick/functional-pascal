# 06 — Re-evaluate `Bridged*` view wrappers after TV 2.0

**Status:** [ ] Not started · [ ] In progress · [ ] Done

**Priority:** Medium — after or in parallel with [done/04-headless-test-util.md](done/04-headless-test-util.md)

**Depends on:** turbo-vision 2.0 behavior (radio groups, mouse on clusters, disabled commands)

## Problem

FPAS wraps upstream widgets to mirror state into VM cells and to patch input gaps:

| Module | Purpose |
| --- | --- |
| `bridged_check_box.rs` | Left-click toggle + `TurboVisionBoolCell` |
| `bridged_radio_button.rs` | Left-click select + group exclusivity |
| `bridged_list_box.rs` | Selection cell sync |
| `check_box_mouse.rs`, `radio_button_mouse.rs` | Mouse handlers when not using bridged path |
| `test_mouse.rs` | Headless coordinate hit-test (separate from live) |

Turbo Vision 2.0 release notes claim: mutually exclusive radio groups, mouse-clickable clusters, disabled-command enforcement, UTF-8 safety. Some bridge code may be obsolete or duplicative.

Standard message boxes ([done/03-about-message-box.md](done/03-about-message-box.md)) bypass `Bridged*` wrappers entirely — upstream `helpers::msgbox` constructs stock views inside `message_box`.

## Target

For each wrapper, document **keep**, **shrink**, or **delete**:

- If upstream handles behavior, use stock TV types in `tv_views.rs` and read state after events or via existing read-back intrinsics.
- If FPAS still needs cells for `ExecDialog` read-back or headless, keep minimal glue only.

## Tasks

- [ ] **Matrix** — Table in this file: widget × behavior × upstream 2.0 support × bridge file × decision.
- [ ] **CheckBox** — Compare live + headless tests with stock `CheckBox` only; try removing `BridgedCheckBox` behind cfg if tests pass.
- [ ] **RadioButton** — Same; verify group id exclusivity matches FPAS `GroupId`.
- [ ] **ListBox** — Verify selection sync and Enter/double-click command dispatch.
- [ ] **Mouse** — Consolidate live vs headless input after [04](done/04-headless-test-util.md) follow-up.
- [ ] **Delete** — Remove unused `*_mouse.rs` or bridged modules per matrix.
- [ ] **Tests** — `tui_turbo_vision_check_box_mouse_test.fpas`, `radio_button_mouse_test.fpas`, radio/list tests.
- [ ] **Context** — Update [00-context.md](00-context.md) bridge table.

## Verification

```text
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_check_box_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_radio_button_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_check_box_mouse_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_radio_button_mouse_test.fpas
```

## Decision matrix (fill during work)

| Widget | FPAS need | TV 2.0 | Current bridge | Decision |
| --- | --- | --- | --- | --- |
| CheckBox | `Checked`, click toggle | TBD | `bridged_check_box.rs` | TBD |
| RadioButton | `Selected`, groups | TBD | `bridged_radio_button.rs` | TBD |
| ListBox | `ListSelection`, command | TBD | `bridged_list_box.rs` | TBD |

## Notes

- Do not remove cells if `ExecDialog` read-back still depends on them without an upstream read API.
- Reference upstream CHANGELOG “Controls” section for 2.0.0 when filling TBD.
