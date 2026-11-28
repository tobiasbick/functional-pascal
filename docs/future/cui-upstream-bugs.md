# CUI upstream bugs

The `CUI-NNN` identifiers are local references used by the FPAS adapters.
Keep the identifiers stable.

<a id="cui-019"></a>

## CUI-019 — Large constant chart ranges produce non-finite geometry

- Upstream: [PR #64](https://github.com/yappologistic/CUI/pull/64) proposes a fix for the same axis-scaling defect. The [maintainer's adoption report](https://github.com/yappologistic/CUI/pull/64#issuecomment-6883945781) excludes the logic and robustness fixes. The native regression still confirms non-finite geometry at `f9597a3` (v1.5.0) on Windows and Linux under WSL.

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

<a id="cui-020"></a>

## CUI-020 — Removing a visible tag drops hidden tags

- Upstream: [Issue #166](https://github.com/yappologistic/CUI/issues/166) reports the defect. [PR #170](https://github.com/yappologistic/CUI/pull/170) proposes a fix.

At CUI revision `6864776`, the tag editor stores more than 64 tags when
`max_tags` is zero. It displays the first 64. Removing a visible tag also
discards the hidden tags. A direct check with 65 short tags leaves 63 after
one Backspace in an empty draft.

The [FPAS tag input](../ui/gui/inputs/tag-input.md) limits the list to 64
visible tags. The host applies the same limit to native edits.

<a id="cui-021"></a>

## CUI-021 — A multiline password field displays its text

- Upstream: [Issue #167](https://github.com/yappologistic/CUI/issues/167) reports the defect. [PR #171](https://github.com/yappologistic/CUI/pull/171) proposes a fix.

At CUI revision `6864776`, `cui_textarea` draws plain text when its shared
`password` option is set. The setting adds a reveal button to the field.

The [FPAS text area](../ui/gui/inputs/text-area.md) accepts plain text.
Use the [single-line password field](../ui/gui/inputs/text-field.md) for
masked input.

<a id="cui-022"></a>

## CUI-022 — The gallery loses focus when Next becomes Finish

- Upstream: [Issue #168](https://github.com/yappologistic/CUI/issues/168) reports the defect. [PR #172](https://github.com/yappologistic/CUI/pull/172) proposes a fix.

At CUI revision `6864776`, activate Next with Enter on the Invite team step.
The gallery replaces the caption with Finish. CUI derives the button ID from
the caption, so a second Enter does not activate Finish until focus returns.

The [FPAS Navigation page](../ui/gui/navigation/README.md#navigation-gallery-page)
uses a stable button ID and keeps focus when the caption changes.

<a id="cui-023"></a>

## CUI-023 — The initial tab selection can be outside the viewport

- Upstream: [Issue #169](https://github.com/yappologistic/CUI/issues/169) reports the defect. [PR #173](https://github.com/yappologistic/CUI/pull/173) proposes a fix.

At CUI revision `6864776`, draw a narrow underline strip with four long captions.
Set the initial index to three. The selected tab stays outside the viewport
after layout. CUI records the index before it has the dimensions required
to scroll to the tab.

After layout, switching away and back scrolls the selected tab into view.
The FPAS binding uses the native viewport. The gallery starts on the first tab.

<a id="cui-024"></a>

## CUI-024 — Long tooltips can end in incomplete UTF-8

At CUI revision `6864776`, the tooltip copies at most 255 bytes from its text.
The byte limit can cut a character that uses more than one byte.

The [FPAS tooltip](../ui/gui/overlays/tooltip.md) checks the UTF-8 length
before rendering. The same limit applies to button tooltip text and its
shortcut caption together.
The [FPAS source list](../ui/gui/ai/source-list.md) ends the URL tooltip at
a complete character and preserves the source URL.

<a id="cui-025"></a>

## CUI-025 — A close call inside its popover scope can leave it open

At CUI revision `6864776`, the gallery calls `cui_popover_close` from inside
the Filters popover. The popover adds a scope to the ID stack. The close call
looks up a different ID in that scope and leaves the original popover open.
A direct check confirms that the same call closes the popover after the scope ends.
The Smart inputs sample also appends an emoji and leaves its popover open.

In the [FPAS example](../ui/gui/overlays/popover.md), Apply sets the popover
visibility in the application model and closes the native panel.
The [FPAS emoji picker example](../ui/gui/smart-inputs/emoji-picker.md) sets
visibility in the model when it inserts an emoji.

<a id="cui-026"></a>

## CUI-026 — A narrow progress fill can extend beyond its track

At CUI revision `6864776`, a solid progress fill can be wider than its track
when the track is narrower than its height. CUI uses the height as the
minimum fill width.

The [FPAS progress bar](../ui/gui/feedback/progress-bar.md) clips the fill to
the track in this case. The same guard applies to the native meter.

<a id="cui-027"></a>

## CUI-027 — Enlarging a virtual list can leave the new area empty

At CUI revision `6864776`, the virtual list determines its visible range
before the new height is laid out. With 100 rows of 36 units, increasing
the height from 120 to 620 units without input still builds rows 0 through 5
and leaves the lower area empty. CUI reports no redraw pending. On the next
frame the list builds rows 0 through 19.

The [FPAS virtual list](../ui/gui/data-display/virtual-list.md) requests a
follow-up frame when the viewport height changes after range selection.

<a id="cui-028"></a>

## CUI-028 — A single sample puts the line chart crosshair at the window edge

At CUI revision `6864776`, the line chart computes the horizontal crosshair
position only for series with at least two samples. Hovering a chart with one
sample draws its crosshair at the window's left edge.

The [FPAS line chart](../ui/gui/charts/line-chart.md) supplies the horizontal
origin of the native plot when the chart has one sample.

<a id="cui-029"></a>

## CUI-029 — A pattern meter can retain an obsolete selection

At CUI revision `6864776`, the pattern meter can keep a keyboard selection
that refers to a removed part. The readout can use the obsolete selection
when the parts change.

The [FPAS pattern meter](../ui/gui/charts/pattern-meter.md) adjusts the retained
selection to the new part count before drawing. An empty meter clears it.

<a id="cui-030"></a>

## CUI-030 — Bars can extend outside a fixed value range

At CUI revision `6864776`, bars can paint beyond the plot when their values
fall outside the fixed value range. For the [FPAS bar chart](../ui/gui/charts/bar-chart.md),
use the automatic range or choose limits that include all bar endpoints.

<a id="cui-031"></a>

## CUI-031 — A toggle can lose keyboard focus when its glyph changes

At CUI revision `6864776`, the theme toggle changes its glyph and tooltip
when the appearance changes. CUI derives the button ID from those values.
The next Enter key does not activate the toggle until focus returns.

The [FPAS theme toggle](../ui/gui/actions/theme-toggle.md) keeps one stable ID
for both appearances. [Canvas controls](../ui/gui/canvas-and-bars/canvas-controls.md)
also keep their toggle IDs when the glyph or tooltip changes.

<a id="cui-032"></a>

## CUI-032 — A scope at the ID stack limit can close its parent

At CUI revision `6864776`, a push at the ID stack limit leaves the stack
unchanged. A matching pop can then remove the caller's active scope.

The [FPAS swipe adapter](../ui/gui/actions/swipe-actions.md) checks the remaining
capacity before opening its native scopes.

<a id="cui-033"></a>

## CUI-033 — The split chevron can reopen its menu on the same click

At CUI revision `6864776`, pressing the chevron of an open split menu can
close the popup as an outside press. Releasing the chevron reopens it.

The [FPAS split button](../ui/gui/actions/split-button.md) consumes the release
of that closing click.

<a id="cui-034"></a>

## CUI-034 — A disabled split button can keep accepting menu input

At CUI revision `6864776`, an open split menu can accept Enter after the
split button becomes disabled. The gallery uses enabled split buttons.

The [FPAS split button](../ui/gui/actions/split-button.md) closes the menu when
the control becomes disabled.

<a id="cui-035"></a>

## CUI-035 — The Actions save example keeps the Saved caption after its timeout

At CUI revision `6864776`, the Actions page draws the Saved state before it
updates the model to Idle. The page stops requesting frames at the same time.
Saved can remain visible until input requests a new frame.

The [FPAS status button example](../ui/gui/actions/status-button.md) sends a
timed message that updates the model and wakes the view at each transition.

<a id="cui-036"></a>

## CUI-036 — Undo can restore a canceled inline edit

At CUI revision `6864776`, Escape cancels an inline edit and leaves the undo
history attached to the field. Undo can restore the canceled text when the
same field opens again.

The [FPAS inline editor](../ui/gui/smart-inputs/inline-edit.md) clears the
undo history when it discards the native edit session.

<a id="cui-037"></a>

## CUI-037 — A short emoji history can split a UTF-8 character

At CUI revision `6864776`, the recent emoji buffer can truncate a selected
emoji that exceeds the available capacity. The buffer may end inside a
UTF-8 character.

The [FPAS emoji picker](../ui/gui/smart-inputs/emoji-picker.md) keeps the
history intact when the selected emoji cannot fit. The selection still reaches
the application.

<a id="cui-038"></a>

## CUI-038 — Shortcut recording can drop a modifier released in the same frame

At CUI revision `6864776`, the recorder reads the current modifier state
when it consumes a pending key press. If the modifier is released before
that frame, the recorder can store the key without the modifier.

The [FPAS shortcut recorder](../ui/gui/smart-inputs/shortcut-recorder.md)
uses the native input semantics.

<a id="cui-039"></a>

## CUI-039 — Extremely small scrub steps can jump to a range endpoint

At CUI revision `6864776`, native fine scrubbing divides the value by the
fine increment before rounding. An extreme ratio can exceed the range of
single-precision arithmetic and send the value to the wrong endpoint.

The [FPAS scrub field](../ui/gui/smart-inputs/scrub-field.md) validates the
step and range before using the native calculation.

<a id="cui-040"></a>

## CUI-040 — Clicking a money field can retain grouping in its draft

At CUI revision `6864776`, pointer focus can leave thousands separators in
the editable draft. Keyboard focus through Tab removes them. This changes
which character Delete removes at the same cursor position.

The [FPAS money field](../ui/gui/smart-inputs/money-field.md) opens an
ungrouped decimal draft for both focus paths and restores grouping on blur.

<a id="cui-041"></a>

## CUI-041 — A small time dial can select the wrong hour

At CUI revision `6864776`, a 24-hour dial with a small diameter can place
the inner hour ring on the opposite side of the center from its hit targets.
The [FPAS time dial](../ui/gui/pickers/time-dial.md) uses a minimum explicit
diameter of 160 logical units.

<a id="cui-042"></a>

## CUI-042 — Space in the multi-select search can change an item

At CUI revision `6864776`, Space can enter a character in the search field
and toggle the highlighted checkbox in the same frame. The
[FPAS multi-select](../ui/gui/pickers/multi-select.md) sends Space to the
search editor while it has focus.

<a id="cui-043"></a>

## CUI-043 — Enter can change a selection while the popup closes

At CUI revision `6864776`, Enter can toggle a highlighted item in the
multi-select popup during its closing animation. The
[FPAS multi-select](../ui/gui/pickers/multi-select.md) stops accepting
selection input when the popup starts to close. Enter can also commit a
model choice while the [composer menu](../ui/gui/voice/composer.md) closes
or after the composer becomes disabled. FPAS leaves the model selection
alone in both cases.

<a id="cui-044"></a>

## CUI-044 — A second calendar can replace the beginning of a range

At CUI revision `6864776`, the calendar tracks endpoint selection per
control. When two controls share a range, a second control can interpret
the next selection as a new start. The
[FPAS date range calendar](../ui/gui/pickers/date-range-calendar.md) reads
the pending start from the shared range so the next selection completes it.

<a id="cui-045"></a>

## CUI-045 — Two months can show the same month for one frame

At CUI revision `6864776`, navigating back in the two-month range calendar
from December and January can render December twice during the activation
frame. The next frame shows November and December. FPAS uses the native
[range calendar](../ui/gui/pickers/date-range-calendar.md) rendering sequence.

<a id="cui-046"></a>

## CUI-046 — A clipped file target can accept a drop outside its visible area

At CUI revision `6864776`, a dropzone or avatar uses the full control bounds
for file drag feedback and drop handling. A clipped part can still respond.
FPAS uses the visible bounds of the [dropzone](../ui/gui/capture/dropzone.md)
and [avatar upload](../ui/gui/capture/avatar-upload.md) for file drag input.

<a id="cui-047"></a>

## CUI-047 — Browse activation can mask an accepted avatar drop

At CUI revision `6864776`, a valid drop can replace the avatar texture in the
same frame as a browse activation. CUI can return the browse result after
the replacement. The [FPAS avatar upload](../ui/gui/capture/avatar-upload.md)
reports the accepted drop when the texture changes.

<a id="cui-048"></a>

## CUI-048 — Enter can confirm a question after focus leaves the popover

At CUI revision `6864776`, Enter can accept a confirmation after Tab moves
focus to another control. The [FPAS confirm popover](../ui/gui/disclosure/confirm-popover.md)
lets the focused control receive Enter. The confirm and cancel buttons
accept their own keyboard activation.

<a id="cui-049"></a>

## CUI-049 — A hidden part can retain keyboard focus

At CUI revision `6864776`, changing the text so it fits its line limit can
leave focus on the missing link. If the link returns, Space can activate it.
Palette swatches, Shuffle and the seed lock can also keep focus while hidden.
[FPAS expandable text](../ui/gui/disclosure/expandable-text.md),
[palette strip](../ui/gui/creative-controls/palette-strip.md) and
[seed field](../ui/gui/creative-controls/seed-field.md) release focus when
the part disappears. The [FPAS voice session](../ui/gui/voice/voice-session.md)
releases focus when its Mute or End button disappears.

<a id="cui-050"></a>

## CUI-050 — A focused inline link has no visual focus indicator

At CUI revision `6864776`, Tab can give a link focus without changing its
appearance. The [FPAS link](../ui/gui/disclosure/link.md) draws the native
focus ring around the focused link.

<a id="cui-051"></a>

## CUI-051 — A held pointer can request repeated hover card frames

At CUI revision `6864776`, holding the pointer button suppresses card
opening. Once the delay passes, CUI can still request an immediate frame.
The [FPAS hover card](../ui/gui/disclosure/hover-card.md) waits for eligible
input before scheduling another frame for the pending open.

<a id="cui-052"></a>

## CUI-052 — The banner action does not show keyboard focus

At CUI revision `6864776`, Tab focuses the banner action. The action paints
its hover underline and has no keyboard focus indicator. The
[FPAS banner](../ui/gui/disclosure/banner.md) draws a focus ring on the action.

<a id="cui-053"></a>

## CUI-053 — The focus ring covers a completed checklist step marker

At CUI revision `6864776`, keyboard focus on a completed step can hide its
circle and check mark. The [FPAS checklist](../ui/gui/disclosure/onboarding-checklist.md)
keeps the completion marker visible with the focus ring.

<a id="cui-054"></a>

## CUI-054 — A jump can land at the wrong offset during a scroll

At CUI revision `6864776`, outline navigation can combine an anchor from
the last layout with a viewport offset from the current frame. If scrolling
also moves the viewport in that frame, the jump can miss its heading. The
[FPAS table of contents](../ui/gui/reader/table-of-contents.md) uses the
same scroll offset for the anchor position and the jump calculation.

<a id="cui-055"></a>

## CUI-055 — The outline can highlight the old section after a jump

At CUI revision `6864776`, a jump with reduced motion can move the article
and leave the active marker on the old section until another frame. The
[FPAS table of contents](../ui/gui/reader/table-of-contents.md) updates the
marker and requests a frame to settle the outline.

<a id="cui-056"></a>

## CUI-056 — A row checkbox can lose the range selection origin

At CUI revision `6864776`, a row checkbox can set the internal range origin
after CUI publishes the supplied anchor value. A controlled grid can apply
the old anchor on the next frame. The [FPAS data grid](../ui/gui/data-grid/data-grid.md)
publishes the final anchor after submitting the rows.

<a id="cui-057"></a>

## CUI-057 — An initial grid cursor can stay outside the viewport

At CUI revision `6864776`, a grid can accept an initial cursor beyond the
realized rows and leave the viewport at the top. The
[FPAS data grid](../ui/gui/data-grid/data-grid.md) brings the cursor into view
when it first receives a cursor without an explicit scroll target.

<a id="cui-058"></a>

## CUI-058 — A restored tree chevron can activate from an old press

At CUI revision `6864776`, a chevron can disappear while pressed and return
before release. Release can then toggle the restored chevron. The
[FPAS tree cell](../ui/gui/data-grid/tree-cell.md) releases the old press
when the cell becomes a leaf.

<a id="cui-059"></a>

## CUI-059 — Enter can edit a hidden grid column

At CUI revision `6864776`, the native editor can start in a priority-hidden
column when Enter activates a row. The [FPAS data grid](../ui/gui/data-grid/data-grid.md)
chooses a visible editable cell and ends a draft if its column becomes hidden.

<a id="cui-060"></a>

## CUI-060 — Undo can restore a discarded grid draft

At CUI revision `6864776`, undo history can survive after Escape cancels a
grid edit. Undo in a later session can restore the earlier draft. The
[FPAS data grid](../ui/gui/data-grid/data-grid.md) retires the native editor and
its undo history when a session closes.

<a id="cui-061"></a>

## CUI-061 — The graphical filter value list requires a pointer

At CUI revision `6864776`, a focused filter chip opens the value list with
Enter. The value rows accept pointer input. Arrow keys and Enter do not
select a value. The [FPAS GUI filter bar](../ui/gui/data-grid/filter-bar.md)
uses the native list.

<a id="cui-062"></a>

## CUI-062 — A column added to a live grid can stay at zero width

At CUI revision `6864776`, the first layout initializes column widths. A
later column can retain a zero width. The
[FPAS data grid](../ui/gui/data-grid/data-grid.md) initializes each new
column from its requested width or the CUI default. Existing resized
columns keep their widths.

<a id="cui-063"></a>

## CUI-063 — Plain code can omit indented lines

At CUI revision `6864776`, the plain code renderer can skip a line that
starts with indentation. The [FPAS code block](../ui/gui/code/code-block.md)
draws those lines in the native code viewport.

<a id="cui-064"></a>

## CUI-064 — Mixed tab and space indentation can shift code

At CUI revision `6864776`, leading tabs mixed with spaces can render
at a different position from the equivalent space indentation. The
[FPAS code block](../ui/gui/code/code-block.md) expands tabs for rendering.
The [tool call](../ui/gui/ai/tool-call.md) uses the same rendering for its
payload. Copy preserves the original source bytes.

<a id="cui-065"></a>

## CUI-065 — Code blocks and JSON viewers can stay at the old scroll offset

At CUI revision `6864776`, replacing the content of a scrolled code block
or JSON viewer can leave the new content outside the viewport until
another frame. The [FPAS code block](../ui/gui/code/code-block.md) and
[JSON viewer](../ui/gui/code/json-viewer.md) request a frame to bring the
scroll offset within the new content bounds.

<a id="cui-066"></a>

## CUI-066 — An empty card stack can receive keyboard focus

At CUI revision `6864776`, an empty card stack creates a focusable target
without a visible card. The [FPAS card stack](../ui/gui/social-and-media/card-stack.md)
skips the empty target in Tab navigation.

<a id="cui-067"></a>

## CUI-067 — A pointer drag and an arrow key can dismiss two cards

At CUI revision `6864776`, an arrow key can dismiss a card during a pointer
drag. Releasing the pointer can dismiss the next card. The
[FPAS card stack](../ui/gui/social-and-media/card-stack.md) uses the active
pointer gesture until release.

<a id="cui-068"></a>

## CUI-068 — A disabled image viewer can consume wheel input

At CUI revision `6864776`, a disabled image viewer can intercept the wheel
inside a scrollable container. The
[FPAS image viewer](../ui/gui/social-and-media/image-viewer.md) leaves wheel
input to the surrounding container when disabled.

<a id="cui-069"></a>

## CUI-069 — The minimum crop size can break a fixed aspect ratio

At CUI revision `6864776`, the cropper can clamp each side independently.
Near the minimum size, the resulting rectangle can have a different aspect
ratio. The [FPAS cropper](../ui/gui/social-and-media/image-cropper.md)
keeps the requested ratio when it applies the size limits.

<a id="cui-070"></a>

## CUI-070 — Hovering an avatar can leave the presence row collapsed

At CUI revision `6864776`, an avatar can receive hover input while the
presence row remains collapsed. The
[FPAS presence row](../ui/gui/social-and-media/presence.md) expands while a
visible avatar receives hover input and retains the avatar's tooltip.

<a id="cui-071"></a>

## CUI-071 — The card stack has no visible keyboard focus

At CUI revision `6864776`, the top card can receive keyboard focus with no
visible indicator. The [FPAS card stack](../ui/gui/social-and-media/card-stack.md)
draws the native focus ring around the top card for keyboard focus.

<a id="cui-072"></a>

## CUI-072 — A counter can split a UTF-8 character

At CUI revision `6864776`, the counter sends each byte of an affix to the
text renderer separately. The [FPAS counter](../ui/gui/social-and-media/counter.md)
keeps the bytes of each UTF-8 character together during rendering.

<a id="cui-073"></a>

## CUI-073 — A short ticker can leave a blank region

At CUI revision `6864776`, the ticker builds three copies of its items.
A short set can leave space to the right of the strip. The
[FPAS ticker](../ui/gui/social-and-media/ticker.md) repeats the set to cover
the visible area.

<a id="cui-074"></a>

## CUI-074 — A carousel slide can retain its previous width

At CUI revision `6864776`, the carousel uses the viewport width from the
previous frame. A change to the container width can leave the active slide
at the old size until another frame. The
[FPAS carousel](../ui/gui/social-and-media/carousel.md) requests another
frame when layout changes the viewport width.

<a id="cui-075"></a>

## CUI-075 — A hovered child can leave carousel auto-advance active

At CUI revision `6864776`, an interactive child can receive hover input
while the carousel continues to advance. The
[FPAS carousel](../ui/gui/social-and-media/carousel.md) pauses over its
slides and keeps the child's input active.

<a id="cui-076"></a>

## CUI-076 — Resting shadows can retain the previous theme strength

At CUI revision `6864776`, a switch from light to dark keeps the light
theme's resting shadow. The lower cards of the Social and media deck and
the Motion player use that shadow. The
[FPAS theme](../ui/gui/host.md#system-appearance) applies the dark theme's
resting shadow, which matches a dark theme at startup.

<a id="cui-077"></a>

## CUI-077 — A week calendar drag can lose its final movement

At CUI revision `6864776`, the week calendar can ignore a pointer move and
release delivered in one frame. The
[FPAS week calendar](../ui/gui/planning/week-calendar.md) applies the final
pointer position before it ends the drag.

<a id="cui-078"></a>

## CUI-078 — Weekday captions can disagree with the start date

At CUI revision `6864776`, the week calendar always starts its weekday
captions on Monday. The dates come from the supplied start date.
The [FPAS week calendar](../ui/gui/planning/week-calendar.md) aligns the
captions with that date, including localized names.

<a id="cui-079"></a>

## CUI-079 — Gantt ignores the height option

At CUI revision `6864776`, Gantt uses the combined height of the header and
task rows. The [FPAS Gantt chart](../ui/gui/planning/gantt.md) uses a scroll
viewport for an explicit height. Task captions and bars scroll together.

<a id="cui-080"></a>

## CUI-080 — Month selection can fail for a date before 1970

At CUI revision `6864776`, the month calendar treats negative Unix day
numbers as missing dates. The
[FPAS month calendar](../ui/gui/planning/month-calendar.md) maps the native
calendar dates so selections before 1970 work.

<a id="cui-081"></a>

## CUI-081 — Focusing the month grid can navigate away from the displayed month

At CUI revision `6864776`, the grid can restore a focus day from another
month on Tab. The [FPAS month calendar](../ui/gui/planning/month-calendar.md)
keeps the initial keyboard focus within the displayed month.

<a id="cui-082"></a>

## CUI-082 — Kanban can report a drop for a card that left its source column

At CUI revision `6864776`, the drag retains the source column after the
application removes or transfers the card. The
[FPAS board](../ui/gui/planning/kanban.md) cancels the drag when the card
leaves its source column.

<a id="cui-083"></a>

## CUI-083 — Gantt can retain its old layout after a width change

At CUI revision `6864776`, a container resize can leave the Gantt chart in
its old narrow or wide layout. The [FPAS Gantt chart](../ui/gui/planning/gantt.md)
requests a frame when the measured width crosses the layout threshold.

<a id="cui-084"></a>

## CUI-084 — Dragging a long appointment can produce a negative start time

At CUI revision `6864776`, the week calendar can produce a negative start
time if the event is longer than the visible time range. The
[FPAS week calendar](../ui/gui/planning/week-calendar.md) keeps the event's
hours when it cannot fit within the visible range.

<a id="cui-085"></a>

## CUI-085 — A keyboard lift can retain input after focus moves

At CUI revision `6864776`, a lifted Kanban card can consume Enter after Tab
moves focus to another control. The
[FPAS board](../ui/gui/planning/kanban.md) cancels the keyboard lift when
focus moves away from the card.

<a id="cui-086"></a>

## CUI-086 — A disabled week calendar can consume wheel input

At CUI revision `6864776`, the disabled week calendar can intercept wheel
input. The [FPAS week calendar](../ui/gui/planning/week-calendar.md) leaves
wheel input to the enclosing scroll area when the calendar is disabled.

<a id="cui-087"></a>

## CUI-087 — Heatmap gaps can activate adjacent cells

At CUI revision `6864776`, the heatmap and calendar heatmap can highlight
or activate a cell from the gap next to it. FPAS uses the painted cell
bounds for [heatmap](../ui/gui/more-charts/heatmap.md) and
[calendar heatmap](../ui/gui/more-charts/calendar-heatmap.md) input.

<a id="cui-088"></a>

## CUI-088 — The candlestick cursor can outlive its selected period

At CUI revision `6864776`, replacing the period series can leave a stale
keyboard cursor. The [FPAS candlestick chart](../ui/gui/more-charts/candlestick.md)
keeps the cursor within the current series and clears it when the series
is empty.

<a id="cui-089"></a>

## CUI-089 — Waffle gaps can activate adjacent squares

At CUI revision `6864776`, a pointer in the gap between squares can
highlight or activate a nearby square. The
[FPAS waffle chart](../ui/gui/more-charts/waffle.md) uses the painted square
bounds for pointer input and preserves legend activation.

<a id="cui-090"></a>

## CUI-090 — Funnel ignores the height option

At CUI revision `6864776`, the funnel uses fixed stage and connector
heights for every value of `cui_funnel_opts.height`. The
[FPAS funnel](../ui/gui/more-charts/funnel.md) uses the native stage sizing.

<a id="cui-091"></a>

## CUI-091 — Bar list input can follow the target rank during an animation

At CUI revision `6864776`, a moving row can appear under the pointer while
the native hit uses the destination rank. The
[FPAS bar list](../ui/gui/more-charts/bar-list.md) uses the visible row
and reports its original input index.

<a id="cui-092"></a>

## CUI-092 — A passive radial gauge can intercept card input

At CUI revision `6864776`, a radial gauge registers a hit on its face.
An enclosing card can then miss pointer input. The
[FPAS radial gauge](../ui/gui/more-charts/radial-gauge.md) leaves that input
to the enclosing card.

<a id="cui-093"></a>

## CUI-093 — A wrapped radar legend can overlap the next widget

At CUI revision `6864776`, the radar chart reserves one legend row. Long
series names can wrap below the chart. The
[FPAS radar chart](../ui/gui/more-charts/radar-chart.md) reserves space for
the wrapped legend when the chart width changes.

<a id="cui-094"></a>

## CUI-094 — Activity rings can accept input on an undrawn ring

At CUI revision `6864776`, a ring that has no space to draw can still
receive pointer input inside the chart. The
[FPAS activity rings](../ui/gui/more-charts/activity-rings.md) give pointer
input to visible rings. The legend can still activate the listed goals.

<a id="cui-095"></a>

## CUI-095 — Keyboard focus can replace the candlestick plot

At CUI revision `6864776`, a keyboard focus ring replaces the chart's
paint operation. The focus ring remains visible while the plot and its
crosshair disappear. The
[FPAS candlestick chart](../ui/gui/more-charts/candlestick.md) paints the
native plot and then draws the focus ring.

<a id="cui-096"></a>

## CUI-096 — Loaders keep moving with reduced motion

At CUI revision `6864776`, the loader advances and requests more frames when
reduced motion is on. The [FPAS loader](../ui/gui/motion/loader.md) draws a
still indicator and leaves other widgets' frame requests in place.
The [agent status](../ui/gui/ai/agent-status.md) uses the same still indicator.

<a id="cui-097"></a>

## CUI-097 — The typewriter caret does not blink

At CUI revision `6864776`, the typewriter draws the caret at constant opacity
while text appears. The [FPAS typewriter](../ui/gui/motion/typewriter.md) blinks
the caret while the animation runs, including during the initial delay.

<a id="cui-098"></a>

## CUI-098 — Text reveal groups words across tabs and newlines

At CUI revision `6864776`, only ASCII spaces start a new step in the text
reveal. A tab or newline can group adjacent words into one step. The
[FPAS text reveal](../ui/gui/motion/text-reveal.md) separates these steps and
preserves the original whitespace in the text.

<a id="cui-099"></a>

## CUI-099 — Keyboard focus can replace the waveform

At CUI revision `6864776`, a keyboard focus ring replaces the waveform paint
operation. The [FPAS waveform](../ui/gui/motion/waveform.md) paints the peaks
and draws the focus ring over them. The [FPAS voice waveform](../ui/gui/voice/voice-waveform.md)
and [voice note](../ui/gui/voice/voice-note.md) also keep both paint operations.

<a id="cui-100"></a>

## CUI-100 — A dragged control can lose the pointer release position

At CUI revision `6864776`, a pointer move and release between frames can leave
the control at the previous position. The
[FPAS waveform](../ui/gui/motion/waveform.md),
[player](../ui/gui/motion/now-playing.md), [knob](../ui/gui/audio/knob.md),
[fader](../ui/gui/audio/fader.md) and
[event track](../ui/gui/instruments/event-track.md) use the recorded release
position to complete the gesture. The
[ratio](../ui/gui/creative-controls/ratio-slider.md),
[gradient](../ui/gui/creative-controls/gradient-slider.md) and
[step](../ui/gui/creative-controls/step-slider.md) sliders also use that
position. The [node editor](../ui/gui/canvas-and-bars/node-editor.md) also
applies the pointer movement that arrives with a pan release.
The [voice waveform](../ui/gui/voice/voice-waveform.md) and
[voice note](../ui/gui/voice/voice-note.md) also retain the final seek position
when a click or drag ends between frames.

<a id="cui-101"></a>

## CUI-101 — A static text gradient changes direction

At CUI revision `6864776`, a static text gradient reaches the second color
near the middle and returns toward the first color. The
[FPAS gradient text](../ui/gui/motion/gradient-text.md) blends from the first
color toward the second across the text.

<a id="cui-102"></a>

## CUI-102 — A disabled spotlight can still respond to the pointer

At CUI revision `6864776`, a spotlight can draw a pointer glow inside a
disabled scope. The [FPAS spotlight](../ui/gui/motion/spotlight-card.md) checks
whether it can accept pointer input before drawing the hover effect.

<a id="cui-103"></a>

## CUI-103 — A still spotlight can keep requesting frames

At CUI revision `6864776`, a beamless spotlight with reduced motion can
request frames while the pointer stays still over it. The
[FPAS spotlight](../ui/gui/motion/spotlight-card.md) stops requesting hover
frames once the glow settles.

<a id="cui-104"></a>

## CUI-104 — The tab bar selection can stay at its old position after resize

At CUI revision `6864776`, a tab bar can draw its selection indicator at the
old position when the parent changes width. The
[FPAS tab bar](../ui/gui/motion/tab-bar.md) requests a frame when its geometry
changes so the indicator moves to the selected tab.

<a id="cui-105"></a>

## CUI-105 — The dock can overflow when it first appears

At CUI revision `6864776`, the first dock frame uses the full icon size
before the layout accounts for the item count. The
[FPAS dock](../ui/gui/motion/dock.md) uses the current entries to size the
first frame.

<a id="cui-106"></a>

## CUI-106 — Dock padding can keep requesting frames without magnification

At CUI revision `6864776`, a compact dock can request frames while the pointer
stays still over its padding, even when magnification is off. The
[FPAS dock](../ui/gui/motion/dock.md) leaves the dock still and preserves the
wake-up time for item labels.

<a id="cui-107"></a>

## CUI-107 — A covered widget can activate from the keyboard

At CUI revision `6864776`, Enter or Space can activate a focused control
behind an open popover. This affects
[prompt suggestions](../ui/gui/ai/prompt-suggestions.md),
[status tags](../ui/gui/creative-controls/status-tag.md),
[aspect options](../ui/gui/creative-controls/aspect-ratio-picker.md),
[option actions](../ui/gui/creative-controls/option-action.md),
[palette swatches](../ui/gui/creative-controls/palette-strip.md),
[seed fields](../ui/gui/creative-controls/seed-field.md) and the tools in
[gradient](../ui/gui/creative-controls/gradient-slider.md) and
[step](../ui/gui/creative-controls/step-slider.md) sliders. This also affects
[canvas controls](../ui/gui/canvas-and-bars/canvas-controls.md),
[result cards](../ui/gui/canvas-and-bars/result-card.md),
[selection bars](../ui/gui/canvas-and-bars/selection-bar.md),
[toolbar buttons](../ui/gui/canvas-and-bars/toolbar.md),
[voice notes](../ui/gui/voice/voice-note.md) and
[voice sessions](../ui/gui/voice/voice-session.md).
FPAS leaves those keys to the popover.

<a id="cui-108"></a>

## CUI-108 — A tool call can reopen when its payload returns

At CUI revision `6864776`, an open tool call keeps its expanded state after
both payloads disappear. If a payload returns, the panel opens without input.
The [FPAS tool call](../ui/gui/ai/tool-call.md) clears expansion when it has no
payload.

<a id="cui-109"></a>

## CUI-109 — A composer can send against an earlier draft

At CUI revision `6864776`, the composer can process Enter or a Send click
before it applies the same frame's edit. This can miss a new draft or send
an empty one. The
[FPAS composer](../ui/gui/voice/composer.md) checks the edited draft before
submitting.

<a id="cui-110"></a>

## CUI-110 — A recorder can lose its height during a take

At CUI revision `6864776`, a composer with a wrapped draft can shrink during
a recording when the hidden editor's geometry expires. The
[FPAS composer](../ui/gui/voice/composer.md) retains the editor width while
recording.

<a id="cui-111"></a>

## CUI-111 — An agent plan connector can follow old row positions

At CUI revision `6864776`, the plan draws its connector from the preceding
frame's rows. The connector can stay out of place after a row wraps until
another frame runs. The [FPAS agent plan](../ui/gui/ai/agent-plan.md) uses the
row positions from the current layout.

<a id="cui-112"></a>

## CUI-112 — A source card can receive focus outside the visible list

At CUI revision `6864776`, Tab can focus a clipped card without scrolling it
into view. The [FPAS source list](../ui/gui/ai/source-list.md) scrolls to show
the focused card.

<a id="cui-113"></a>

## CUI-113 — A disabled recorder can consume Escape

At CUI revision `6864776`, a previously focused recorder can consume Escape
and propose cancel within a disabled scope. The
[FPAS composer](../ui/gui/voice/composer.md) leaves Escape available to the
enclosing controls when the recorder cannot accept input.

<a id="cui-114"></a>

## CUI-114 — A source icon can cut a non-ASCII character

At CUI revision `6864776`, the source list and citation preview use only the
first byte of a domain name for its icon. The
[FPAS source list](../ui/gui/ai/source-list.md) and
[citation](../ui/gui/ai/citation.md) draw the complete first character.

<a id="cui-115"></a>

## CUI-115 — A removed meter channel can still reset a clip

At CUI revision `6864776`, changing from two channels to one can retain a clip
from the discarded channel. A pointer release can report a reset for this
hidden clip. The [FPAS level meter](../ui/gui/audio/level-meter.md) retires
removed channels and cancels the reset gesture when no clip remains.

<a id="cui-116"></a>

## CUI-116 — A stepped audio control can stop short of its upper bound

At CUI revision `6864776`, End snaps to a lower step when the upper bound
falls between steps. The [FPAS knob](../ui/gui/audio/knob.md) and
[fader](../ui/gui/audio/fader.md) apply the upper bound directly for End.

<a id="cui-117"></a>

## CUI-117 — A mirrored level meter can place its clip light at the quiet end

At CUI revision `6864776`, a right-to-left container mirrors the clip light
while the horizontal bars still increase from left to right. The
[FPAS level meter](../ui/gui/audio/level-meter.md) keeps its internal layout
aligned with the scale and follows the parent layout direction as a unit.

<a id="cui-118"></a>

## CUI-118 — A wheel event can reach the previous audio control after Tab

At CUI revision `6864776`, Tab can move focus while a wheel event in the same
frame still changes the knob or fader that had focus. The
[FPAS knob](../ui/gui/audio/knob.md) and [fader](../ui/gui/audio/fader.md)
use the current focus for wheel input.

<a id="cui-119"></a>

## CUI-119 — A dimmed control can consume guided tour navigation keys

At CUI revision `6864776`, a dimmed fader can retain focus. Right can then
change the fader and advance the tour. The
[FPAS guided tour](../ui/gui/overlays/guided-tour.md) routes keys to its own
controls or to the active target within the spotlight.

<a id="cui-120"></a>

## CUI-120 — Wheel input can scroll through the guided tour scrim

At CUI revision `6864776`, a wheel event over the dimmed area can scroll
the underlying content. The
[FPAS guided tour](../ui/gui/overlays/guided-tour.md) blocks this input on
the scrim.

<a id="cui-121"></a>

## CUI-121 — A file browser can keep a listing after its options change

At CUI revision `6864776`, changing Hidden or MaximumEntries in the same
folder can retain the old listing. A refresh can still use the previous
entry limit. The [FPAS file browser](../ui/gui/files/file-browser.md)
applies both options to the current listing request.

<a id="cui-122"></a>

## CUI-122 — A disabled terminal can accept a shortcut

At CUI revision `6864776`, a terminal with retained input focus can handle
Ctrl+L inside a disabled scope. The
[FPAS terminal](../ui/gui/files/terminal.md) applies the input scope before
processing terminal shortcuts.

<a id="cui-123"></a>

## CUI-123 — A terminal can initially wrap at the wrong width

At CUI revision `6864776`, a newly inserted terminal can use its fallback
width to wrap output after an idle frame. The
[FPAS terminal](../ui/gui/files/terminal.md) requests another frame when
the laid-out output width changes.

<a id="cui-124"></a>

## CUI-124 — A filtered-out file can remain selected

At CUI revision `6864776`, a browser can keep a selection after a filter
excludes its row. An Open picker can still accept that path. The
[FPAS browser](../ui/gui/files/file-browser.md) and
[picker](../ui/gui/files/file-picker.md) use the visible selection.

<a id="cui-125"></a>

## CUI-125 — A tree menu can use the pointer position after the press

At CUI revision `6864776`, a right press and a move to another row in the
same frame can open the menu for the later row. The
[FPAS tree menu](../ui/gui/data-grid/tree-table.md) uses the native
right-click input path and has the same batching limit.

<a id="cui-126"></a>

## CUI-126 — A terminal can ignore ANSI italic styling

At CUI revision `6864776`, the terminal recognizes the ANSI italic command
and draws the text with the regular glyphs. The
[FPAS terminal](../ui/gui/files/terminal.md) uses the same native painter.

<a id="cui-127"></a>

## CUI-127 — A disabled file picker can accept a browser shortcut

At CUI revision `6864776`, F5 can refresh the file browser inside a
disabled picker. The [FPAS picker](../ui/gui/files/file-picker.md)
applies the enabled state to browser shortcuts as well as dialog actions.

<a id="cui-128"></a>

## CUI-128 — A disabled trace viewer can still zoom

At CUI revision `6864776`, the trace viewer can change its visible interval
in response to the wheel while disabled. The
[FPAS trace viewer](../ui/gui/developer-tools/trace-viewer.md) applies its
enabled state to wheel input.

<a id="cui-129"></a>

## CUI-129 — A waterfall timing bar can miss its tooltip

At CUI revision `6864776`, hovering a timing bar reports the request index
and can omit the timing tooltip. The
[FPAS network waterfall](../ui/gui/developer-tools/network-waterfall.md)
lets the native tooltip track the hovered bar.

<a id="cui-130"></a>

## CUI-130 — A dependency node can activate when disabled during a press

At CUI revision `6864776`, disabling the dependency graph during a press can
leave an activation pending on release. The
[FPAS dependency graph](../ui/gui/developer-tools/dependency-graph.md)
clears the held gesture when the control is disabled.

<a id="cui-131"></a>

## CUI-131 — A focused widget can handle keys beneath an open popup

At CUI revision `6864776`, a focused widget can handle selection or
activation keys while a popup covers it. This affects the
[trace viewer](../ui/gui/developer-tools/trace-viewer.md),
[network waterfall](../ui/gui/developer-tools/network-waterfall.md),
[dependency graph](../ui/gui/developer-tools/dependency-graph.md),
[commit graph](../ui/gui/developer-tools/commit-graph.md),
[pipeline](../ui/gui/developer-tools/pipeline.md),
[problems panel](../ui/gui/developer-tools/problems-panel.md),
[keymap](../ui/gui/developer-tools/keymap.md),
[test explorer](../ui/gui/more-developer-tools/test-explorer.md),
[stack trace](../ui/gui/more-developer-tools/stack-trace.md),
[API explorer](../ui/gui/more-developer-tools/api-explorer.md),
[event track](../ui/gui/instruments/event-track.md) and
[request bar](../ui/gui/more-developer-tools/http-request-bar.md).
The covered request URL and [composer](../ui/gui/voice/composer.md) can also
accept queued text. FPAS leaves that input to the popup.
The [aspect](../ui/gui/creative-controls/aspect-ratio-picker.md) and
[option](../ui/gui/creative-controls/option-action.md) pickers and the
[palette](../ui/gui/creative-controls/palette-strip.md) also leave arrow
keys to the popup, as do [toolbar radio groups](../ui/gui/canvas-and-bars/toolbar.md).

<a id="cui-132"></a>

## CUI-132 — A commit graph can initially use the wrong column layout

At CUI revision `6864776`, a newly mounted narrow commit graph can draw
the wide author and date columns and stay in that layout until another
frame runs. The [FPAS commit graph](../ui/gui/developer-tools/commit-graph.md)
requests a frame when the actual width differs from the layout width.

<a id="cui-133"></a>

## CUI-133 — A widget can retain its layout after resize

At CUI revision `6864776`, the pipeline, problems panel and keymap can retain
their old layout after their parent changes width. The
[FPAS pipeline](../ui/gui/developer-tools/pipeline.md),
[problems panel](../ui/gui/developer-tools/problems-panel.md) and
[keymap](../ui/gui/developer-tools/keymap.md) request a frame when their
laid-out width changes. The [test explorer](../ui/gui/more-developer-tools/test-explorer.md),
[stack trace](../ui/gui/more-developer-tools/stack-trace.md),
[blame view](../ui/gui/more-developer-tools/blame.md),
[API explorer](../ui/gui/more-developer-tools/api-explorer.md),
[feature flags](../ui/gui/more-developer-tools/feature-flags.md) and
[request bar](../ui/gui/more-developer-tools/http-request-bar.md) also
request a frame when their width changes. A
[result card](../ui/gui/canvas-and-bars/result-card.md) also requests a frame
if its image height was calculated from the previous width. The
[composer](../ui/gui/voice/composer.md) requests another frame when the
laid-out width changes its wrapped draft height.

<a id="cui-134"></a>

## CUI-134 — A query can report a change to an empty choice every frame

At CUI revision `6864776`, a choice field with an empty first option can
report a change on idle frames while the condition already has that value.
The [FPAS query builder](../ui/gui/developer-tools/query-builder.md) emits
a change when the edited query actually differs.

<a id="cui-135"></a>

## CUI-135 — A schedule preset can be shortened to a different schedule

At CUI revision `6864776`, a short expression buffer can truncate a preset
with several weekdays to its first weekday. The
[FPAS schedule picker](../ui/gui/developer-tools/schedule-picker.md) retains
the previous expression if the complete edit does not fit.

<a id="cui-136"></a>

## CUI-136 — A disabled developer widget can still accept wheel input

At CUI revision `6864776`, the wheel can scroll a disabled diagnostic list,
test explorer or blame view. The
[FPAS problems panel](../ui/gui/developer-tools/problems-panel.md),
[test explorer](../ui/gui/more-developer-tools/test-explorer.md) and
[blame view](../ui/gui/more-developer-tools/blame.md) block wheel input
while disabled.

<a id="cui-137"></a>

## CUI-137 — A question mark in a URL fragment can start a query

At CUI revision `6864776`, the URL helpers can treat a question mark inside
a fragment as the query delimiter. Replacing the query can then duplicate
part of the fragment. The [FPAS query parameter functions](../ui/gui/more-developer-tools/http-request-bar.md#query-parameters)
identify the fragment before they pass the query to CUI.

<a id="cui-138"></a>

## CUI-138 — URL replacement can truncate a fragment or reject an exact fit

At CUI revision `6864776`, URL replacement can truncate a fragment longer
than 255 bytes. It can also reject an unescaped byte even when the result
fits. The [FPAS query parameter functions](../ui/gui/more-developer-tools/http-request-bar.md#query-parameters)
preserve the complete fragment and check the length of the encoded result.

<a id="cui-139"></a>

## CUI-139 — Keyboard input can reach a hidden endpoint

At CUI revision `6864776`, the API explorer can move the selection into a
folded group. Enter can also toggle an endpoint that the current filter
excludes. The [FPAS API explorer](../ui/gui/more-developer-tools/api-explorer.md)
limits keyboard navigation and activation to visible endpoints.

<a id="cui-140"></a>

## CUI-140 — A regex match chip can omit its final character

At CUI revision `6864776`, a match chip can omit the last character when
that character uses multiple UTF-8 bytes. The
[FPAS regex tester](../ui/gui/more-developer-tools/regex-tester.md) keeps
complete characters in match labels.

<a id="cui-141"></a>

## CUI-141 — An open rollout popup can keep an obsolete anchor

At CUI revision `6864776`, a change to the rollout percentage can replace
the popup trigger while the popup stays open. Resizing can leave the popup
at the old position. Removing the environment can also leave it open.
The [FPAS feature flag editor](../ui/gui/more-developer-tools/feature-flags.md)
follows the current trigger and closes the popup when the environment goes away.

<a id="cui-142"></a>

## CUI-142 — Coverage can retain a path after the caller replaces its data

At CUI revision `6864776`, the coverage tree caches caller-owned paths.
The [FPAS coverage adapter](../ui/gui/more-developer-tools/coverage.md)
refreshes the cached paths when it supplies the current frame's data.

<a id="cui-143"></a>

## CUI-143 — Coverage hides children beyond the per-folder limit

At CUI revision `6864776`, the coverage display includes up to 512 children
per folder. The aggregate can include children beyond this display limit.
The [FPAS coverage widget](../ui/gui/more-developer-tools/coverage.md)
uses the same native tree.

<a id="cui-144"></a>

## CUI-144 — Sorting coverage can move selection to a different file

At CUI revision `6864776`, sorting can keep the same selected row index
after the rows change order. The
[FPAS coverage widget](../ui/gui/more-developer-tools/coverage.md) keeps the
selection on the same file when it sorts the rows.

<a id="cui-145"></a>

## CUI-145 — Uptime can label a historical end date as Today

At CUI revision `6864776`, uptime uses the last date in its tooltips and
always labels the end of the axis Today. The start label counts all bars,
including the end bar. The [FPAS uptime widget](../ui/gui/more-developer-tools/uptime.md)
uses the native axis labels.

<a id="cui-146"></a>

## CUI-146 — Copying a stack trace can omit text and frames

At CUI revision `6864776`, CUI allocates the clipboard buffer without
counting the heading and message lengths. A long message can fill the
buffer and leave no space for the frames. The
[FPAS stack trace](../ui/gui/more-developer-tools/stack-trace.md) includes
the full heading, message and frames in the copied text.

<a id="cui-147"></a>

## CUI-147 — A replacement stack trace can retain stale expansion state

At CUI revision `6864776`, changing the source locations or previews while
keeping the function names can retain the old expansion state. The
[FPAS stack trace](../ui/gui/more-developer-tools/stack-trace.md) updates
the expansion state when the trace changes.

<a id="cui-148"></a>

## CUI-148 — Enter can open a test excluded by the filter

At CUI revision `6864776`, the test explorer can retain the selected test
when a filter hides its row. Enter can then open the hidden test. The
[FPAS test explorer](../ui/gui/more-developer-tools/test-explorer.md)
limits activation to a visible test.

<a id="cui-149"></a>

## CUI-149 — A request bar action can lose focus when its label changes

At CUI revision `6864776`, the request bar can retain keyboard focus on the
Send button after the application switches to Cancel. A second Enter can
then fail to cancel. The
[FPAS request bar](../ui/gui/more-developer-tools/http-request-bar.md) moves
the action focus to the button for the new loading state.

<a id="cui-150"></a>

## CUI-150 — The Instruments gallery can label zero cups as 5+

At CUI revision `6864776`, removing the final cup leaves the CUPS readout
at 5+. The [FPAS Instruments page](../ui/gui/instruments/README.md)
displays 0 for an empty track.

<a id="cui-151"></a>

## CUI-151 — The dot-matrix chart omits series names from its readout

At CUI revision `6864776`, the dot-matrix chart accepts a name for each
series and omits the names from the readout. The
[FPAS dot-matrix chart](../ui/gui/instruments/dot-chart.md) passes the names
to CUI and uses the native readout.

<a id="cui-152"></a>

## CUI-152 — A click outside a dot chart plot can activate the chart

At CUI revision `6864776`, clicking the value gutter or the axis label area
can report an activation with a missing sample index. The
[FPAS dot-matrix chart](../ui/gui/instruments/dot-chart.md) restricts pointer
activation to the plot.

<a id="cui-153"></a>

## CUI-153 — Readout Left and Right reverse in right-to-left layout

At CUI revision `6864776`, the readout treats physical Left and Right as
logical start and end. In right-to-left layout this places the content on
the opposite side. The [FPAS readout](../ui/gui/instruments/readout.md)
keeps the physical side and the reading direction.

<a id="cui-154"></a>

## CUI-154 — A scrolling dot display can stop requesting frames

At CUI revision `6864776`, the dot display can use an obsolete width after
mounting or resizing. Displays in separate anonymous containers can also
share an identity. The [FPAS dot-matrix display](../ui/gui/instruments/dot-display.md)
gives each display a stable ID and schedules the next frame from its current
width.

<a id="cui-155"></a>

## CUI-155 — An explicit label interval can fail to advance the axis

At CUI revision `6864776`, floating-point rounding can leave an axis tick
at the same position after adding the explicit interval. The
[FPAS dot-matrix chart](../ui/gui/instruments/dot-chart.md) and
[event track](../ui/gui/instruments/event-track.md) reject intervals that
cannot advance across the axis.

<a id="cui-156"></a>

## CUI-156 — A choice can move against the arrow in RTL

At CUI revision `6864776`, the option field moves by logical index
in right-to-left layout. Left can leave the selection on the visually
rightmost option. The [FPAS option field](../ui/gui/creative-controls/option-action.md)
and [toolbar radio groups](../ui/gui/canvas-and-bars/toolbar.md) move in the
visual direction of the arrow.

<a id="cui-157"></a>

## CUI-157 — The focus ring can stay on the old choice for a frame

At CUI revision `6864776`, an arrow can change the focused choice while
the ring stays on the old one for the current frame. The
[FPAS aspect ratio picker](../ui/gui/creative-controls/aspect-ratio-picker.md),
[FPAS option field](../ui/gui/creative-controls/option-action.md),
[FPAS palette strip](../ui/gui/creative-controls/palette-strip.md) and
[FPAS toolbar radio groups](../ui/gui/canvas-and-bars/toolbar.md) draw the
ring on the new focus target.

<a id="cui-158"></a>

## CUI-158 — Editing a ratio caption can remove keyboard focus

At CUI revision `6864776`, the ratio slider uses both captions to compute
its identity. Changing either caption can leave focus on the old identity.
The [FPAS ratio slider](../ui/gui/creative-controls/ratio-slider.md) uses
the supplied stable widget ID for input.

<a id="cui-159"></a>

## CUI-159 — An active seed animation can ignore reduced motion

At CUI revision `6864776`, switching to reduced motion during a dice roll
can leave the old digit animation running. The
[FPAS seed field](../ui/gui/creative-controls/seed-field.md) settles the
display when reduced motion takes effect.

<a id="cui-160"></a>

## CUI-160 — Presence tooltips can use the wrong hover state

At CUI revision `6864776`, a lone named avatar can show its tooltip at an
unrelated pointer position. Multiple avatars can reset a hovered avatar's
tooltip timer and request frames while the pointer is idle elsewhere. The
[FPAS presence pill](../ui/gui/canvas-and-bars/presence-pill.md) submits the
native tooltip for the hovered avatar.

<a id="cui-161"></a>

## CUI-161 — Gallery Fit can leave a moved node out of view

At CUI revision `6864776`, the Canvas gallery calculates Fit bounds from
the first and last nodes' horizontal positions and a fixed height. A node
moved outside those bounds can stay outside the viewport after Fit. The
[FPAS gallery](../ui/gui/canvas-and-bars/README.md) includes the current
bounds of every sample node and keeps the original rectangle at startup.

<a id="cui-162"></a>

## CUI-162 — A format color palette can open without keyboard focus

At CUI revision `6864776`, a custom palette can open without a focused
swatch if it omits the current color. Enter then selects nothing. The
[FPAS format bar](../ui/gui/canvas-and-bars/format-bar.md) focuses the first
swatch in that case.

<a id="cui-163"></a>

## CUI-163 — A disabled format bar can confirm deletion

At CUI revision `6864776`, disabling a format bar while its Delete
confirmation is open can leave the dialog active. Enter can still confirm
the action. The [FPAS format bar](../ui/gui/canvas-and-bars/format-bar.md)
closes its confirmation and menus when disabled or removed.

<a id="cui-164"></a>

## CUI-164 — A literal double hash can shorten a style caption

At CUI revision `6864776`, the format bar style menu interprets `##` in
a custom style caption as a native identity suffix. It hides the rest of
the caption. The [FPAS format bar](../ui/gui/canvas-and-bars/format-bar.md)
displays the full style name.

<a id="cui-165"></a>

## CUI-165 — Custom zoom presets can clamp an idle canvas

At CUI revision `6864776`, a custom preset range can clamp an existing
viewport even without input. Reset can also clamp 100 percent to that
range. [FPAS canvas controls](../ui/gui/canvas-and-bars/canvas-controls.md)
leave the idle viewport alone and reset to 100 percent.

<a id="cui-166"></a>

## CUI-166 — Zoom can skip closely spaced presets

At CUI revision `6864776`, a fixed tolerance can treat two distinct zoom
presets as the same value. [FPAS canvas controls](../ui/gui/canvas-and-bars/canvas-controls.md)
step through consecutive representable preset values.

<a id="cui-167"></a>

## CUI-167 — Cycling a single toolbar choice reports a change

At CUI revision `6864776`, activating a one-choice cycle returns true
even though the selected index stays the same. The
[FPAS toolbar](../ui/gui/canvas-and-bars/toolbar.md) reports a cycle change
only when the index changes.

<a id="cui-168"></a>

## CUI-168 — A pending toolbar choice can apply after disabling

At CUI revision `6864776`, a radio choice made in the overflow menu can
reach the owner on a later frame even if the group has since been disabled.
The [FPAS toolbar](../ui/gui/canvas-and-bars/toolbar.md) discards that
pending choice.

<a id="cui-169"></a>

## CUI-169 — A narrow format bar can hide color or style controls

At CUI revision `6864776`, the color or style trigger can disappear at
very small widths without a corresponding overflow item. The
[FPAS format bar](../ui/gui/canvas-and-bars/format-bar.md) documents the
available space requirement for these native controls.

<a id="cui-170"></a>

## CUI-170 — A node can draw its old selection outline for one frame

At CUI revision `6864776`, pressing a node submitted later in the frame
can leave the selection outline on a node already submitted. The
[FPAS node editor](../ui/gui/canvas-and-bars/node-editor.md) applies the
final selection to the outlines and reports it to the model.

<a id="cui-171"></a>

## CUI-171 — A voice note can exceed the available width

At CUI revision `6864776`, a voice note enforces a minimum width of
200 logical points. It can exceed its parent or a smaller fixed width.
The [FPAS voice note](../ui/gui/voice/voice-note.md) fits the width offered
by the parent and gives the playback controls the available space.

<a id="cui-172"></a>

## CUI-172 — A zero-duration voice note can report a seek

At CUI revision `6864776`, clicking a zero-duration note's waveform
can report Seek with position still zero. The
[FPAS voice note](../ui/gui/voice/voice-note.md) treats a zero-duration
waveform as display-only.

<a id="cui-173"></a>

## CUI-173 — A composer can cancel and send the same recording

At CUI revision `6864776`, Escape and Enter in one recorder frame can
report both Cancel and Voice Send. The
[FPAS composer](../ui/gui/voice/composer.md) reports Cancel only in this
case.

<a id="cui-174"></a>

## CUI-174 — Leaving the window can discard an earlier button release

At CUI revision `6864776`, leaving the window after releasing a pressed
button can discard the click when both events reach the same frame. The
leave event clears the pointer hit before the button evaluates the pending
release. The voice note's Play button reproduces the issue with native
input calls.

<a id="cui-175"></a>

## CUI-175 — A compact status item can show its native ID in the tooltip

At CUI revision `6864776`, a compact status bar can replace an item's
caption with its icon. Its automatically generated tooltip can include the
caption's `###` identity suffix. The
[FPAS status bar](../ui/gui/app-chrome/status-bar.md) supplies the visible
caption as the tooltip.

<a id="cui-176"></a>

## CUI-176 — A full clip stack can drop the parent canvas clip

At CUI revision `6864776`, CUI ignores a clip push when the stack is full.
A matching pop can then remove a valid outer clip, so drawing can extend
past the canvas. [FPAS canvas painting](../ui/gui/canvas-and-bars/painting.md)
checks capacity before adding a clip and pops only the clips it accepts.

<a id="cui-177"></a>

## CUI-177 — The emoji picker filters by the neutral skin tone

At CUI revision `6864776`, `cui_emoji_picker_ex` checks the neutral glyph
when it builds the list. The picker hides an emoji if the font lacks the neutral
glyph, even if the font supports the selected skin tone. In Segoe UI Emoji on
Windows, searching for `woman astronaut` with the medium skin tone reproduces
the issue. The same search succeeds with Noto Color Emoji on Linux.

The [FPAS emoji picker test](../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiEmojiPickerTests.cs)
uses the rainbow flag to check that a short history still returns the complete
selection without losing the previous entry.
