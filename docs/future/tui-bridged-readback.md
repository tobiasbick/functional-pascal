# Std.Tui: three upstream read-back blockers

**Status:** the Turbo Vision bridge refactor is complete except for these three adapters.
**Upstream pin:** `turbo-vision` git tag `v2.0.0`.

## Blockers

| Upstream view | FPAS adapter | Required read-back |
| --- | --- | --- |
| `CheckBox` | `bridge/bridged_check_box.rs` | checked state |
| `RadioButton` | `bridge/bridged_radio_button.rs` | selected index |
| `OutlineViewer` | `bridge/bridged_outline.rs` | selection and selected text |

At the pinned upstream revision, these concrete views cannot be downcast from `dyn View` after `handle_event`; `View` has no supported `as_any_mut`-style hook. The adapters therefore synchronize keyboard and mouse changes into FPAS host cells. Removing them earlier would regress `CheckBox.Checked`, `RadioButton.Selected`, `Outline.Selection`, or `Outline.SelectedText`.

All other TUI widgets use direct upstream views. The current API is documented under [docs/pascal/std/tui/](../pascal/std/tui/README.md).

## Periodic upstream check

Before touching TUI/VM code or declaring the refactor closed:

1. Compare upstream releases with the `v2.0.0` pin in the root `Cargo.toml`.
2. Check whether `CheckBox`, `RadioButton`, and `OutlineViewer` have documented live read-back or downcasting through `dyn View`.
3. If available, bump the dependency and complete the closure steps below.

## Closure steps

1. Replace the three adapter constructions in `bridge/views/{check_box,radio_button,outline}.rs` with direct upstream views.
2. Delete the three `bridged_*.rs` files and their declarations in `bridge/mod.rs`.
3. Run:

   ```bash
   cargo test -p fpas-vm bridge::
   fpas test tests/tui/views/check_box_test.fpas
   fpas test tests/tui/views/radio_button_test.fpas
   fpas test tests/tui/views/outline_test.fpas
   fpas test tests/tui/views/outline_selection_test.fpas
   fpas test tests/tui/events/check_box_mouse_test.fpas
   fpas test tests/tui/events/radio_button_mouse_test.fpas
   rg bridged_ crates/  # expect no matches
   ```

4. Remove this file's blocker notice and the archive reference once the commands are green.

## Suggested upstream issue

> **Title:** Expose live read-back for `CheckBox`, `RadioButton`, and `OutlineViewer` embedders
>
> FPAS embeds turbo-vision views behind opaque Pascal handles. After keyboard/mouse input, we need to sync external state for checkbox checked state, radio selection, and outline selection/text. ListBox and other types already support this pattern; checkbox, radio, and outline require `View::as_any_mut` (or an equivalent documented read-back API) on those concrete types so embedders can update host cells after `handle_event`.
