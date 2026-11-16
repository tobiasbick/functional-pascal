# CUI upstream bugs

The `CUI-NNN` identifiers are local references used by the FPAS adapters.
Keep the identifiers stable.

## CUI-001–CUI-005 — Upstream links

- [Issue #58](https://github.com/yappologistic/CUI/issues/58)
- [PR #59](https://github.com/yappologistic/CUI/pull/59)
- Fixes: [`01badea`](https://github.com/yappologistic/CUI/commit/01badea),
  [`bbb9ab8`](https://github.com/yappologistic/CUI/commit/bbb9ab8),
  [`73aab56`](https://github.com/yappologistic/CUI/commit/73aab56)
- Verified snapshot: [`86d9781`](https://github.com/yappologistic/CUI/commit/86d9781)
- [PR confirmation](https://github.com/yappologistic/CUI/pull/59#issuecomment-6849961418)
- [Issue confirmation](https://github.com/yappologistic/CUI/issues/58#issuecomment-6850153025)

<a id="cui-019"></a>

## CUI-019 — Large constant chart ranges produce non-finite geometry

- Upstream: [PR #64](https://github.com/yappologistic/CUI/pull/64) proposes a fix for the same axis-scaling defect. The [maintainer's adoption report](https://github.com/yappologistic/CUI/pull/64#issuecomment-6883945781) excludes the logic and robustness fixes. The native regression still confirms non-finite geometry at `a5c9ef5`.

Reproduced on Linux with direct `cui_line_chart_ex` calls at `86d9781`.
Use one series containing three `1e10f` samples with automatic bounds and reduced
motion. After `cui_end_frame`, all 20 emitted vertices have non-finite positions.
The automatic range expands a constant by one; that addition cannot advance
this single-precision value. The resulting zero range makes the tick step and
projected geometry invalid. Very small spans and overflowing stacked totals
also require validation before painting.

The FPAS binding checks that axis ticks can advance within a finite range and
rejects unrepresentable data before calling CUI. An explicit wider range such
as zero through `2e10f` renders the constant series using native CUI. This keeps
the native scaling algorithm unchanged for representable ranges. The native
chart test retains the direct NaN reproduction without rasterizing it; binding
tests cover rejection, stacked overflow and the explicit-range alternative.

<a id="cui-032"></a>

## CUI-032 — Upstream links

- [Issue #75](https://github.com/yappologistic/CUI/issues/75)
- [PR #76](https://github.com/yappologistic/CUI/pull/76) · [PR #93](https://github.com/yappologistic/CUI/pull/93)
- Fix: [`5830981`](https://github.com/yappologistic/CUI/commit/5830981)
- Verified snapshot: [`a5c9ef5`](https://github.com/yappologistic/CUI/commit/a5c9ef5efc39b3fc783758362f6b8f9f130f5883)
- [Upstream confirmation](https://github.com/yappologistic/CUI/pull/93#issuecomment-6907821736)

<a id="cui-033"></a>

## CUI-033 — Planning drag resumes after editing or its target disappears

Reproduced on Windows using direct `cui_gantt` and `cui_week_calendar` calls
with the vendored snapshot at `a5c9ef5`.

Start moving the second task or appointment with the left mouse button held.
Set `read_only` for a frame, then restore editing while the button is still
held. In Gantt, temporarily reducing the task count from two to one has the
same effect. Continue moving the pointer before releasing the button. CUI
resumes the previous drag and changes the task dates or appointment times.
Restoring editability or the task should require a new press to start a drag.

The FPAS adapters clear the active drag when editability changes, input is
disabled, or the dragged index is outside the current item count. A click
still activates a task or appointment in steady read-only mode. The native
planning regression checks all three transitions after first observing a
successful move.
