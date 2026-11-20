# CUI gaps

The [CUI widget inventory](../cui-widget-inventory.md) records accepted widgets
and current terminal coverage. Desktop host and publishing work is tracked in
[Desktop GUI application gaps](gui-app-gaps.md). Confirmed defects with FPAS
workarounds are tracked in [CUI upstream bugs](cui-upstream-bugs.md).

## Findings awaiting upstream reporting

The following findings were checked against `a5c9ef5`. CUI-033 also reproduces
at `71c383f`. Follow the [contribution workflow](../cui-contributions.md) before
publishing a report or fix.

- [ ] Wheel regions ignore paint transforms. `cui__paint_wheel_hit`
  (`src/cui_layout.c:1076`) registers the rectangle in layout coordinates, but
  intersects it with a transformed clip. This affects `cui_wheel_region` and
  nine built-in callers (dials, emoji, flame, graph, photo, terminal, viz). A
  chart in a dragged `.draggable` dialog would take the wheel at its old place.
  Inferred from code; reproduce it with a canvas in a moved dialog before
  reporting. The fix belongs in `cui__paint_wheel_hit`.
- [ ] Custom axis labels lose small values. `cui__ins_label`
  (`src/cui_instrument.c:38`, from `731a4c6`) passes 0 to the label function
  for any value below `1e-9`, so a callback that scales seconds to nanoseconds
  shows 0. Only negative zero needs normalizing. Report it as a comment on
  [#28](https://github.com/yappologistic/CUI/issues/28), which added the label
  functions.
- [ ] An editable week-calendar drag that ends between frames keeps the event
  at the last frame's position; the motion up to the release is lost and the
  click is suppressed. Older than #99, raised in its
  [review](https://github.com/yappologistic/CUI/pull/99#discussion_r4603195287).
- [ ] SDL2 placement sizes under `CUI_SCALE` with the software renderer differ
  from `cui_app_size`. Deferred in the
  [#95 review](https://github.com/yappologistic/CUI/pull/95#discussion_r4580381315).
- [ ] [CUI-033](cui-upstream-bugs.md#cui-033) has an FPAS workaround but no
  upstream report yet.
- Not to report: shared-library test linkage (`CUI_BUILD_SHARED=ON` with tests)
  affects five test programs that call private `cui__` functions through
  `cui::cui` (`cui_tests`, `cui_test_devtools`, `cui_test_dither`,
  `cui_component_shots`, `cui_rtl_layout`; inferred, not built) and has no CI
  job; already explained in
  [#97](https://github.com/yappologistic/CUI/pull/97#issuecomment-6955446066).
  `cui_test_inflate` and the bidi programs compile their sources directly
  ([correction](https://github.com/yappologistic/CUI/pull/97#issuecomment-6956800280)).

## Strict gallery pixel parity

- [ ] Remove the remaining single-channel antialiasing differences in wide page
  composition. At `86d9781`, Linux software captures at 1240x1700, scale 1,
  differ by one 8-bit channel step at 182/184 pixels in the Inputs picker card
  (dark/light), 487 in dark Feedback progress and one in dark Loading. The
  measured wide picker positions differ by about 0.00012 logical points from
  direct CUI. Ten of twelve reviewed Inputs regions and four of six reviewed
  Feedback regions match exactly. Comparisons exclude the reserved sample email,
  unsynchronized activity animations and the unbound skeleton group. The native
  widget and calendar tests compare at identical positions and times and pass
  without a pixel tolerance. Keep strict whole-page parity open until the
  composition differences are resolved.

- [ ] Match the supplementary App chrome composition to CUI's documentation
  captures. At 1100 and 550 logical points, scale one, Software and OpenGL show
  remaining differences in the editor code and status alignment, the Docking
  viewport illustration and frame, and the boundary pixels of Split view.
  The direct native widget comparison is exact. Keep the assembled examples
  open until their corresponding regions also match.
