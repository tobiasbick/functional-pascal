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

- Upstream: [PR #64](https://github.com/yappologistic/CUI/pull/64) proposes a fix for the same axis-scaling defect. Retest this reproduction on upstream `master` once the PR is resolved.

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

## CUI-032 — A read-only week event can activate after a drag on release

- Upstream: [Issue #75](https://github.com/yappologistic/CUI/issues/75) · [PR #76](https://github.com/yappologistic/CUI/pull/76)
- Follow-up: [PR #93](https://github.com/yappologistic/CUI/pull/93) adds the release-frame drag check and regression test.

The fixes in `6e6b353` restore keyboard and pointer activation in read-only
schedules. One case still reproduces in the native week calendar. Press an
event and submit pointer motion beyond its drag threshold in the same frame
as the release. CUI updates the moved flag only while the button is down, so
it reports an activation. The binding checks motion on release and retains
the test for this sequence.
