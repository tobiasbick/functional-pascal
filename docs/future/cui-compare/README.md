# CUI gallery comparison

The CUI layout integration review is complete.

## Gallery pages

Record the CUI reference commit and link to the comparison evidence for each page.
For pixel comparisons, use the same platform, renderer, window size, display scale,
theme and widget state.

| CUI gallery page | FPAS GUI gallery page | Layout | Pixel match | Behavior | Example coverage | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| [Overview](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L128) | [Overview](../../ui/gui/overview/README.md) | Verified at three widths | Exact for the page body with matching sample text | Verified | All examples present | [Overview evidence](#overview-evidence) |

## CUI functions

Record each reviewed `cui_*` function once. Link to the gallery pages that exercise it.

| CUI function | FPAS GUI API | Integration status | Gallery pages | Evidence |
| --- | --- | --- | --- | --- |
| [`cui_avatar`](../../ui/gui/disclosure/avatar.md) | `Avatars.Avatar` | Adopted | Overview | Team initials and presence indicators |
| [`cui_badge`](../../ui/gui/feedback/badge.md) | `Badges.Badge` | Adopted | Overview | Team role badges and tones |
| [`cui_box`](../../ui/gui/feedback/grid.md) | `Grids.Grid` | Adopted | Overview | Responsive metric and card grids |
| [`cui_button`](../../ui/gui/actions/button.md) | `Buttons.Button` | Adopted | Overview | Browse, Open command palette, Create workspace and Invite |
| [`cui_card`](../../ui/gui/feedback/card.md) | `Cards.Card` | Adopted | Overview | Workspace and Team card surfaces |
| [`cui_checkbox`](../../ui/gui/selection/checkbox.md) | `Checkboxes.Checkbox` | Adopted | Overview | Summary preference also updates the disabled example in Selection |
| [`cui_column`](../../ui/gui/layout/column.md) | `Elements.Column` | Adopted | Overview | Introduction, card headings and team labels |
| [`cui_command_palette`](../../ui/gui/overlays/command-palette.md) | `CommandPalettes.CommandPalette` | Adopted | Overview | Open from the button or Primary+K |
| [`cui_heading`](../../ui/gui/text-and-display/text.md) | `Elements.Styled` with `DisplayTitle` or `Heading` | Adopted | Overview | Level-one and level-four headings |
| [`cui_input`](../../ui/gui/inputs/text-field.md) | `TextFields.TextField` | Adopted | Overview | Shared name and email fields in the [workspace form](../../ui/gui/overview/workspace-form.md) |
| [`cui_metric`](../../ui/gui/overview/metric.md) | `Metrics.Metric` | Adopted | Overview | Three tiles, including a delta and sparkline for Frame time |
| [`cui_row`](../../ui/gui/layout/row.md) | `Elements.Row` | Adopted | Overview | Wrapping actions and padded team rows |
| [`cui_scroll`](../../ui/gui/layout/scroll-view.md) | `Elements.ScrollView` | Adopted | Overview | Browse and palette navigation retain the main scroll position |
| [`cui_scroll_to`](../../ui/gui/layout/scroll-view.md) | `CuiWindow.ScrollTo` | Adopted | Overview | The FPAS gallery resets scroll on sidebar navigation through the scroll view ID |
| [`cui_secondary`](../../ui/gui/text-and-display/text.md) | `Elements.Styled` with `Secondary` | Adopted | Overview | Card descriptions |
| [`cui_set_theme`](../../ui/gui/host.md#system-appearance) | `Desktop` theme provider | Adopted | Overview | Primary+J switches between light and dark |
| [`cui_shortcut`](../../ui/gui/host.md#application-shortcuts) | `Shortcuts.Binding` and the native palette shortcut | Adopted | Overview | Primary+J and Primary+K |
| [`cui_text`](../../ui/gui/text-and-display/formatted-text.md) | `Elements.Styled` and `Elements.FormattedText` | Adopted | Overview | Introduction and single-line team labels with native ellipsis and tooltips |
| [`cui_toast`](../../ui/gui/overlays/toast.md) | `Toasts.Show` | Adopted | Overview | Successful workspace creation |
| [`cui_viewport_size`](../../ui/gui/layout/layout-inspection.md#read-the-bounds) | `Viewport.LogicalWidth` | Adopted | Overview | Desktop sidebar and compact layout |
| [`cui_wants_keyboard`](../../ui/gui/host.md#application-shortcuts) | GUI host shortcut guard | Adopted | Overview | Native editors keep keyboard ownership during Primary+J |

## Overview evidence

The source reference is CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
Capture page index 0 with CUI's `cui_screenshots` and render the FPAS Overview
with the software renderer. Use Windows, 150% scale, neutral accent, English,
and light and dark themes. Capture the initial view at 1240 × 1000,
640 × 1000 and 500 × 1000 logical units.

Use the FPAS heading, introduction, sample emails and email placeholder in the
reference capture. Compare all RGBA channels. At 1240 logical units, compare
pixels from x = 363 to the viewport edge. At the two compact widths, compare
pixels from y = 105 to the viewport edge. All six page-body comparisons have
zero differing pixels.

An additional 500 × 1400 capture includes every stacked Team row. At both
compact widths, using the same header title also gives a full-frame match in
both themes. The desktop sidebar displays the gallery's own name and logo.

The native parity test compares a truncated formatted label and its hover tooltip
against direct CUI calls. The gallery contract drives the workspace controls,
keyboard shortcuts, command palette and scroll navigation with native input.
[Native parity test](../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
[Gallery contract](../../../tests/Fixtures/gui-host/CompletionContract.fpas)

The focused Overview contract passes with the Windows software and OpenGL
renderers. The full Windows software run stops at a range ruler pointer check
that also fails on the unchanged FPAS baseline.
