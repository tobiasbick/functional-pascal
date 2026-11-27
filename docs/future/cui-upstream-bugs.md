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

At CUI revision `6864776`, the tag editor stores more than 64 tags when
`max_tags` is zero. It displays the first 64. Removing a visible tag also
discards the hidden tags. A direct check with 65 short tags leaves 63 after
one Backspace in an empty draft.

The [FPAS tag input](../ui/gui/inputs/tag-input.md) limits the list to 64
visible tags. The host applies the same limit to native edits.

<a id="cui-021"></a>

## CUI-021 — A multiline password field displays its text

At CUI revision `6864776`, `cui_textarea` draws plain text when its shared
`password` option is set. The setting adds a reveal button to the field.

The [FPAS text area](../ui/gui/inputs/text-area.md) accepts plain text.
Use the [single-line password field](../ui/gui/inputs/text-field.md) for
masked input.

<a id="cui-022"></a>

## CUI-022 — The gallery loses focus when Next becomes Finish

At CUI revision `6864776`, activate Next with Enter on the Invite team step.
The gallery replaces the caption with Finish. CUI derives the button ID from
the caption, so a second Enter does not activate Finish until focus returns.

The [FPAS Navigation page](../ui/gui/navigation/README.md#navigation-gallery-page)
uses a stable button ID and keeps focus when the caption changes.

<a id="cui-023"></a>

## CUI-023 — The initial tab selection can be outside the viewport

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

## CUI-031 — The theme toggle loses keyboard focus after an appearance change

At CUI revision `6864776`, the theme toggle changes its glyph and tooltip
when the appearance changes. CUI derives the button ID from those values.
The next Enter key does not activate the toggle until focus returns.

The [FPAS theme toggle](../ui/gui/actions/theme-toggle.md) keeps one stable ID
for both appearances.

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
selection input when the popup starts to close.

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

## CUI-049 — An expandable text link can retain focus when hidden

At CUI revision `6864776`, changing the text so it fits its line limit can
leave focus on the missing link. If the link returns, Space can activate it.
[FPAS expandable text](../ui/gui/disclosure/expandable-text.md) releases
focus when the link disappears.

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
Copy preserves the original source bytes.

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
theme's resting shadow. The lower cards of the Social and media deck use
that shadow. The [FPAS theme](../ui/gui/host.md#system-appearance) applies
the dark theme's resting shadow, which matches a dark theme at startup.
