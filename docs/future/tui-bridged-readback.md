# Std.Tui: upstream read-back for checkbox, radio, and outline

**Status:** blocked on `turbo-vision` v2.0.0 (git tag `v2.0.0`)  
**Branch:** `refactor/tui-try-2`  
**Plan handoff:** [remaining-work.md](../refactor-tui-try-2/remaining-work.md) stream A

## Problem

FPAS try-2 still ships three VM bridge adapters:

- `crates/fpas-vm/src/vm/execute/io/tui/try2/bridged_check_box.rs`
- `crates/fpas-vm/src/vm/execute/io/tui/try2/bridged_radio_button.rs`
- `crates/fpas-vm/src/vm/execute/io/tui/try2/bridged_outline.rs`

At the current upstream pin, `CheckBox`, `RadioButton`, and `OutlineViewer` do not expose a supported live-state read-back hook through `dyn View`. There is no reliable `as_any_mut` downcast after `handle_event`, so the adapters copy keyboard/mouse selection into FPAS host cells. That keeps `CheckBox.Checked`, `RadioButton.Selected`, `Outline.Selection`, and `Outline.SelectedText` correct after interactive input.

Other widgets (`ListBox`, `Button`, `StaticText`, …) already use direct upstream views.

## Done when

1. Upstream adds downcast or a documented read-back API for those three types **or** FPAS bumps to a revision that includes it.
2. Replace adapter construction in `try2/views/{check_box,radio_button,outline}.rs` with direct upstream views.
3. Delete the three `bridged_*.rs` files and their `mod` declarations in `try2/mod.rs`.
4. Re-run regressions:
   - `tests/tui/views/check_box_test.fpas`
   - `tests/tui/views/radio_button_test.fpas`
   - `tests/tui/views/outline_test.fpas`
   - `tests/tui/views/outline_selection_test.fpas`
   - `tests/tui/events/check_box_mouse_test.fpas`
   - `tests/tui/events/radio_button_mouse_test.fpas`
5. `rg bridged_ crates/` returns no matches.
6. Complete [verification.md](../refactor-tui-try-2/verification.md) plan closure (stream D).

## Suggested upstream issue text

> **Title:** Expose live read-back for `CheckBox`, `RadioButton`, and `OutlineViewer` embedders
>
> FPAS embeds turbo-vision views behind opaque Pascal handles. After keyboard/mouse input, we need to sync external state for checkbox checked state, radio selection, and outline selection/text. ListBox and other types already support this pattern; checkbox, radio, and outline require `View::as_any_mut` (or an equivalent documented read-back API) on those concrete types so embedders can update host cells after `handle_event`.

## Verification

```bash
rg bridged_ crates/                    # expect zero after fix
fpas test tests/tui/views/check_box_test.fpas
fpas test tests/tui/events/check_box_mouse_test.fpas
cargo test -p fpas-vm try2::
```
