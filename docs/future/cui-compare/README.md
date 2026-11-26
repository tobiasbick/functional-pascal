# CUI gallery comparison

The CUI layout integration review is complete.

## Gallery pages

Record the CUI reference commit and link to the comparison evidence for each page.
For pixel comparisons, use the same platform, renderer, window size, display scale,
theme and widget state.

| CUI gallery page | FPAS GUI gallery page | Layout | Pixel match | Behavior | Example coverage | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| [Overview](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L128) | [Overview](../../ui/gui/overview/README.md) | Verified at three widths | Exact for the page body with matching sample text | Verified | All examples present | [Overview evidence](#overview-evidence) |
| [Buttons](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L182) | [Buttons](../../ui/gui/actions/README.md#buttons-gallery-page) | Native cards and rows verified at three widths | Exact for the page body with closed and open menus | Verified | All six sections and examples present | [Buttons evidence](#buttons-evidence) |
| [Inputs](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L262) | [Inputs](../../ui/gui/inputs/README.md#inputs-gallery-page) | Native cards and responsive grids at three widths | 20 exact comparisons with matching email. Two edited views differ at the caret. | Verified | All three sections and 14 examples present | [Inputs evidence](#inputs-evidence) |

## CUI functions

Record each reviewed `cui_*` function once. Link to the gallery pages that exercise it.

| CUI function | FPAS GUI API | Integration status | Gallery pages | Evidence |
| --- | --- | --- | --- | --- |
| [`cui_avatar`](../../ui/gui/disclosure/avatar.md) | `Avatars.Avatar` | Adopted | Overview | Team initials and presence indicators |
| [`cui_badge`](../../ui/gui/feedback/badge.md) | `Badges.Badge` | Adopted | Overview | Team role badges and tones |
| [`cui_box`](../../ui/gui/feedback/grid.md) | `Grids.Grid` | Adopted | Overview, Inputs | Responsive metric, card and input grids |
| [`cui_button`](../../ui/gui/actions/button.md) | `Buttons.Button` | Adopted | Overview, Buttons | Workspace actions and the variant, size, icon, state and group examples |
| [`cui_button_group`](../../ui/gui/actions/button-group.md) | `ButtonGroups.ButtonGroup` | Adopted | Buttons | Bold, Italic and Underline share one frame |
| [`cui_card`](../../ui/gui/feedback/card.md) | `Cards.Card` | Adopted | Overview, Buttons, Inputs | Workspace, Team and example cards |
| [`cui_checkbox`](../../ui/gui/selection/checkbox.md) | `Checkboxes.Checkbox` | Adopted | Overview | Summary preference also updates the disabled example in Selection |
| [`cui_color_picker`](../../ui/gui/inputs/color-picker.md) | `ColorPickers.ColorPicker` | Adopted | Inputs | Brand color swatch |
| [`cui_column`](../../ui/gui/layout/column.md) | `Elements.Column` | Adopted | Overview, Buttons, Inputs | Section headings, form captions and field stacks |
| [`cui_command_palette`](../../ui/gui/overlays/command-palette.md) | `CommandPalettes.CommandPalette` | Adopted | Overview | Open from the button or Primary+K |
| [`cui_copy_button`](../../ui/gui/actions/copy-button.md) | `CopyButtons.CopyButton` | Adopted | Buttons | Copy the displayed install command |
| [`cui_date_picker`](../../ui/gui/inputs/date-picker.md) | `DatePickers.DatePicker` | Adopted | Inputs | Due date and its calendar popup |
| [`cui_dialog`](../../ui/gui/overlays/dialog.md) | `Dialogs.Dialog` | Adopted | Buttons | Options > Delete opens the project confirmation |
| [`cui_dialog_footer`](../../ui/gui/overlays/dialog.md) | `ActionFooters.ActionFooter` | Adopted | Buttons | Cancel and Delete project actions |
| [`cui_heading`](../../ui/gui/text-and-display/text.md) | `Elements.Styled` with `DisplayTitle`, `PageTitle` or `Heading` | Adopted | Overview, Buttons, Inputs | Levels one, two and four |
| [`cui_hold_button`](../../ui/gui/actions/hold-button.md) | `HoldButtons.HoldButton` | Adopted | Buttons | A completed hold shows the Deleted toast |
| [`cui_icon_button`](../../ui/gui/actions/icon-button.md) | `IconButtons.IconButton` | Adopted | Buttons | Copy, Share, Like and Delete icons with tooltips |
| [`cui_input`](../../ui/gui/inputs/text-field.md) | `TextFields.TextField` | Adopted | Overview, Inputs | Shared name and email, plus masked password editors |
| [`cui_label`](../../ui/gui/text-and-display/text.md) | `Elements.Styled` with `Label` and `OneTimeCodes.Options.Label` | Adopted | Inputs | Verification code and Brand color captions |
| [`cui_menu`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Menu` | Adopted | Buttons | Labeled Options and icon-only More actions |
| [`cui_menu_item`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Entry.Action` | Adopted | Buttons | Icons, shortcut captions and a danger item |
| [`cui_menu_label`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Entry.Label` | Adopted | Buttons | Project section caption |
| [`cui_menu_separator`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Entry.Separator` | Adopted | Buttons | Line above Delete |
| [`cui_metric`](../../ui/gui/overview/metric.md) | `Metrics.Metric` | Adopted | Overview | Three tiles, including a delta and sparkline for Frame time |
| [`cui_number`](../../ui/gui/inputs/number-field.md) | `NumberFields.NumberField` | Adopted | Inputs | Seats with bounds and step buttons |
| [`cui_otp_input`](../../ui/gui/inputs/one-time-code.md) | `OneTimeCodes.OneTimeCodeInput` | Adopted | Inputs | Six-box verification code and completion |
| [`cui_password_strength`](../../ui/gui/inputs/password-strength.md) | `PasswordStrengthMeters.PasswordStrengthMeter` | Adopted | Inputs | Live estimate under New password |
| [`cui_row`](../../ui/gui/layout/row.md) | `Elements.Row` | Adopted | Overview, Buttons | Wrapping actions, padded team rows and the copy command |
| [`cui_scroll`](../../ui/gui/layout/scroll-view.md) | `Elements.ScrollView` | Adopted | Overview, Buttons, Inputs | Shared scrolling content area |
| [`cui_scroll_to`](../../ui/gui/layout/scroll-view.md) | `CuiWindow.ScrollTo` | Adopted | Overview, Buttons, Inputs | The FPAS gallery resets scroll on sidebar navigation through the scroll view ID |
| [`cui_search`](../../ui/gui/inputs/search-field.md) | `SearchFields.SearchField` | Adopted | Inputs | Search projects with native clear action |
| [`cui_secondary`](../../ui/gui/text-and-display/text.md) | `Elements.Styled` with `Secondary` | Adopted | Overview, Buttons, Inputs | Card and section descriptions |
| [`cui_segmented`](../../ui/gui/actions/segmented-control.md) | `SegmentedControls.SegmentedControl` | Adopted | Buttons | View choices with icons and a small date range selector |
| [`cui_select`](../../ui/gui/inputs/select.md) | `Selects.Select` | Adopted | Inputs | Country and searchable Language lists |
| [`cui_separator`](../../ui/gui/layout/divider.md) | `Elements.Divider` | Adopted | Overview, Buttons, Inputs | Desktop sidebar divider |
| [`cui_set_theme`](../../ui/gui/host.md#system-appearance) | `Desktop` theme provider | Adopted | Overview | Primary+J switches between light and dark |
| [`cui_shortcut`](../../ui/gui/host.md#application-shortcuts) | `Shortcuts.Binding` and the native palette shortcut | Adopted | Overview | Primary+J and Primary+K |
| [`cui_tag_input`](../../ui/gui/inputs/tag-input.md) | `TagInputs.TagInput` | Adopted | Inputs | Editable Topics list |
| [`cui_text`](../../ui/gui/text-and-display/formatted-text.md) | `Elements.Styled` and `Elements.FormattedText` | Adopted | Overview, Buttons | Introduction, single-line team labels and the monospaced install command |
| [`cui_textarea`](../../ui/gui/inputs/text-area.md) | `TextAreas.TextArea` | Adopted | Inputs | Full-width multiline About field |
| [`cui_toast`](../../ui/gui/overlays/toast.md) | `Toasts.Show` | Adopted | Overview, Buttons, Inputs | Workspace, hold, project and code completion feedback |
| [`cui_viewport_size`](../../ui/gui/layout/layout-inspection.md#read-the-bounds) | `Viewport.LogicalWidth` | Adopted | Overview, Buttons, Inputs | Desktop sidebar and compact layout |
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

## Buttons evidence

Capture page index 1 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
Use Windows, the software renderer, 150% scale, English, and the neutral accent.
Capture light and dark at 1240 × 1000, 640 × 1000 and 500 × 1400 logical units.
Repeat at all three widths with a height of 1420 logical units.
At 1240 × 1420, capture Options and More actions open in each theme.
Move the pointer off the menu after activation and let animations settle.

Compare all RGBA channels to the viewport edges. At desktop width, start at
physical x = 363. At compact widths, start at physical y = 105. These bounds
exclude the sidebar or compact header with its gallery name.
All 16 comparisons have zero differing pixels. The taller captures at 1240
and 640 logical units include the full closed dropdown card.
See the [copy button capture](../../ui/gui/actions/copy-button.md)
for the monospaced install command and its native control.

The [Buttons contract](../../../tests/Fixtures/gui-host/ButtonsContract.fpas)
drives the loading and reset buttons, the group, both selectors, and the hold
button with native input. [Dropdown coverage](../../../tests/Fixtures/gui-host/DropdownContract.fpas)
drives both menus. The [shared dialog contract](../../../tests/Fixtures/gui-host/CompletionContract.fpas)
checks Cancel, confirmation and the toast action. See the [widget guides](../../ui/gui/actions/README.md#buttons-gallery-page)
for the example sources and FPAS APIs.

The Buttons and dropdown contracts pass under Windows with software and
OpenGL. The [native action test](../../../src/Fpas.Ui.Gui.Cui/native/tests/link_render.c)
checks the copy callback and hold completion. The full Windows software run
reaches the same range ruler pointer check as the Overview run.

## Inputs evidence

Use page index 2 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The three original sections contain 14 examples from ten widget families.
Set the reference email to the gallery's reserved sample `maya@example.com`.

Capture Windows software rendering at 150% scale, in English with the neutral
accent. Capture light and dark at 1240 × 1000, 640 × 1000 and 500 × 1400 logical
units. Repeat at all three widths with a height of 1420 logical units.
Compare RGBA through the viewport edges, including the scrollbar.
Start at physical x = 363 on desktop and y = 105 at compact widths.
All 12 initial comparisons have zero differing pixels.

Enable reduced motion for interaction captures. At 500 × 1420, scroll five
wheel lines down to show the full Pickers card. Compare from physical
y = 48 through the viewport edges. Both themes match exactly.

At 1240 × 1420, open the Country, Language and Due date popups separately.
All six comparisons match through the viewport edges. Type `abc` in Password to compare
validation. In both themes, 72 pixels differ at the caret's blink phase.

The [text fields contract](../../../tests/Fixtures/gui-host/TextFieldContract.fpas)
checks native editing and shared profile values. The
[credentials contract](../../../tests/Fixtures/gui-host/CredentialFieldsContract.fpas)
checks tags, code completion and edits in both password fields.
The [picker fields contract](../../../tests/Fixtures/gui-host/PickerFieldsContract.fpas)
checks the select and numeric examples. The
[date contract](../../../tests/Fixtures/gui-host/DateCalendarContract.fpas)
and [color contract](../../../tests/Fixtures/gui-host/ColorEmojiContract.fpas)
check their native popups. See the [input guides](../../ui/gui/inputs/README.md)
for widget options and example sources.

All five contracts pass under Windows with software and OpenGL.
The native input regression, smart input and calendar tests also pass.
