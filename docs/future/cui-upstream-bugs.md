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

<a id="cui-006"></a>

## CUI-006 — Expandable text loses keyboard focus after toggling

Reproduced on Linux with direct `cui_expandable_text_ex` calls at `c8baf35`.
Draw text long enough to exceed two collapsed lines. Press Tab to focus
Show more, then Enter to expand. On the next frame, the link becomes Show less
and no longer has focus. Another Enter does not collapse the text; Tab is
needed to focus the link again. Space has the same problem.

The native widget passes the changing caption to `cui_link`, which hashes
the caption into its identity. Its focus key therefore changes from
`hash("Show more") ^ 0x11AC0009` to `hash("Show less") ^ 0x11AC0009` inside
the widget's ID scope. The widget owns the expansion state, so the application
cannot give this internal link a stable identity through its public options.

FPAS retains native behavior and provides `ExpandableTexts.ToggleId(Id)` for
explicit focus requests. No workaround, upstream issue or fix is planned.

<a id="cui-007"></a>

## CUI-007 — A vertical scroll view consumes horizontal swipe input

Reproduced on Windows at `c8baf35`. Draw a swipe row inside a native
`cui_scroll_begin` viewport with `horizontal = false` and enough content to
scroll vertically. Move the pointer onto the row and send `WM_MOUSEHWHEEL`.
The native scroll view clears both wheel axes before the row reads
`scroll_dx`, so the row stays closed.

The FPAS vertical viewport preserves the horizontal axis for its children.
The Windows native regression sends the same window message and checks that
the row reveals its action while the viewport keeps its vertical position.

<a id="cui-008"></a>

## CUI-008 — A speed dial action pops the caller's disabled scope

Reproduced on Windows with direct `cui_speed_dial_ex` calls at `c8baf35`.
Activate a focused action while the dial is open inside an enabled
`cui_push_disabled(ui, false)` scope. The dial returns the action index and
the native disabled stack depth drops by one.

The dial pushes its own scope only if it starts drawing closed, but checks
its current open state when deciding whether to pop. Choosing an action
changes that state during drawing. The FPAS binding supplies a temporary
scope and pops it only if CUI left it on the stack.

<a id="cui-009"></a>

## CUI-009 — A scroll view consumes vertical input before the wheel picker or ruler

Reproduced on Windows at `c8baf35` in the native Pickers gallery. Move the
pointer onto the day wheel in Wheels and dial and send one vertical scroll
line. The day stays at 14. The enclosing native scroll view clears the
event before `cui_wheel_picker_ex` sees it, including when the whole page fits
in the viewport.

The FPAS wheel adapter registers its clipped bounds as a vertical scroll
target. CUI still consumes the event and changes the wheel index. The GUI
gallery regression sends the same input and verifies that the day moves
from 14 to 15.

The ruler has the same routing defect. A native `cui_ruler_slider_ex` at
172 advances to 173 on one downward wheel line when used by itself and
stays at 172 inside a native vertical scroll view. The FPAS ruler adapter
registers its native tape for both wheel axes. The gallery checks that
each axis reaches the tape.

<a id="cui-010"></a>

## CUI-010 — A wheel picker or ruler loses its scale on keyboard focus

Reproduced on Windows at `c8baf35` using a single native wheel picker with
three month names. Press Tab. The focus ring appears, but the selection band
and captions disappear. The focus ring replaces the wheel's paint callback
and its data.

The FPAS wheel adapter supplies the native wheel painter with its inputs
when CUI has installed a focus ring and calls both native painters. The
gallery capture checks that the selected caption stays visible with keyboard
focus.

Pressing Tab on a standalone native ruler reproduces the same paint callback
replacement. The number and unit remain, but the tape and center marker
disappear. The FPAS ruler adapter preserves both native painters. The gallery
checks that the tape stays visible inside the focus ring.

The native onboarding checklist has the same paint replacement at `c8baf35`.
Tab to the completed circle in a checklist with one completed and one pending
step. The focus ring appears, and the green circle and check disappear. The
checklist bridge preserves the circle painter and adds the focus ring. The
Disclosure gallery verifies both in its focused capture.

<a id="cui-011"></a>

## CUI-011 — Ruler wheel snapping can exceed the upper bound

Reproduced on Windows at `c8baf35` with a standalone ruler, bounds 0 to 1.6,
step 1, and current value 1. One downward wheel line yields 2. The native
wheel branch clamps before rounding to a tick, so rounding can move the
result beyond the bound.

The FPAS adapter clamps the proposed value and synchronizes the native
caption and tape in the same frame. The regression holds the model at 1.6,
scrolls farther, and compares the displayed value with the baseline.

<a id="cui-012"></a>

## CUI-012 — Date and color pickers omit their keyboard focus rings

Reproduced on Windows at `c8baf35` with a standalone native date picker.
Press Tab to focus the field. Its pixels are identical to the unfocused
field. Press Enter to confirm that focus reached it and opens the popup.

The FPAS adapter adds CUI's focus ring to the native trigger when keyboard
focus is visible. The Inputs gallery regression compares the field before
and after keyboard focus.

The native color picker has the same behavior. Its swatch is unchanged on
Tab and opens on Enter. The FPAS color picker applies the native focus ring
to the swatch as well.

A standalone CUI link also has no visible change on Tab in a Windows software
capture at `c8baf35`. Enter activates the focused link. The bridge draws
the native focus ring alongside the original underline painter. The
Disclosure gallery checks the focus pixels and link activation.

<a id="cui-013"></a>

## CUI-013 — Color picker retains stale HSV after a model change

Reproduced on Windows at `c8baf35`. Open a native color picker at `#7747FF`,
replace its supplied color with `#FF0000`, then move hue to 120 degrees.
The native picker returns `#47FF47`, using the old saturation. A hue-only
edit of the new fully saturated red should yield `#00FF00`.

The FPAS adapter reconciles CUI's HSV state when the application replaces
the color or rejects a proposal. The Inputs gallery verifies the hue edit
with an updated RGB value and a non-opaque alpha channel.

<a id="cui-014"></a>

## CUI-014 — Avatar image loading fails with Unicode paths on Windows

At `c8baf35` the shell and file dialog pass UTF-8 paths. The default
bundled image decoder opens them through the narrow Windows file API.
A standalone Windows CUI program fails to load a valid PNG named
`画像.png`. The FPAS shell-drop regression reproduced the failure
after CUI successfully delivered the same path to a dropzone.

The FPAS build enables `STBI_WINDOWS_UTF8` for the bundled decoder. The
Windows test drops the Unicode-named PNG on an avatar and then replaces it
with a differently colored image.

<a id="cui-015"></a>

## CUI-015 — Disabled file targets claim shell drops

At `c8baf35` a standalone dropzone returns one accepted file from
`cui_input_file_drop` even inside `cui_push_disabled`. Dropzone and avatar
upload claim through the same native helper, which has no disabled check.

The FPAS adapters hide drop and drag input from a disabled target for the
duration of its draw. The Windows test posts a shell drop and verifies that
a disabled dropzone delivers no file action.

<a id="cui-016"></a>

## CUI-016 — Hover card reopens after Escape over its anchor

At `c8baf35`, keep the pointer over the native @jonas example until
the card opens. Press Escape while leaving the pointer in place. The popup
closes for one frame and reopens because its hover-entry time still meets
the delay.

The FPAS bridge lets the native close complete and waits for the pointer
to leave the anchor before starting the next hover. The Disclosure gallery
keeps the pointer over the link past the delay after pressing Escape and
checks that the card stays closed.

<a id="cui-017"></a>

## CUI-017 — Scrub commits accept invalid numeric drafts

Reproduced with direct `cui_scrub_ex` calls on Linux at `86d9781`.
Focus a bounded scrub with Tab, start typing with Enter, replace the selected
value with `nan` and press Enter. The proposed value becomes NaN, outside the
finite model contract. The commit parses with `strtod` but checks only whether
any prefix was consumed; trailing garbage and non-finite results are accepted.

The FPAS bridge validates the complete committed draft, rejects non-finite
results and restores the previous value and native edit paint. Valid numbers
still use CUI's own clamping, keys and drag behavior. The native Smart inputs
test retains the direct reproduction beside the protected adapter case.

<a id="cui-018"></a>

## CUI-018 — Money input accepts amounts exceeding its display capacity

Reproduced with direct `cui_money_input_ex` calls on Linux at `86d9781`.
Focus the field, select all, and type 63 nines. CUI accepts an amount above
`1e60`. Its 64-byte text buffer must hold digits, grouping separators and the
terminator. The grouping loop can fill that buffer before writing all digits,
silently truncating the displayed amount.

The FPAS bridge rejects a proposed amount when its fixed-precision grouped
representation would exceed the native 63-byte text capacity. It restores the
model amount and CUI's edit paint together. The native regression records the
unprotected result and verifies the protected adapter; ordinary amount rendering
remains pixel-identical to direct CUI calls.

<a id="cui-019"></a>

## CUI-019 — Large constant chart ranges produce non-finite geometry

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

## CUI-020 — A newly disabled candlestick still consumes a queued wheel

Reproduced on Linux at `86d9781` with direct `cui_candlestick_ex` calls.
Hover the chart until its wheel region is registered, queue a wheel event, then
build the next frame inside `cui_push_disabled(ui, true)`. Its viewport still
zooms because the wheel branch checks the old scroll target without checking
whether the widget is disabled. Two otherwise identical disabled frames produce
different pixels when only one receives that queued event.

The FPAS bridge temporarily hides this chart's wheel target while building a
disabled chart, then restores the target without consuming the input. Enabled
wheel zoom, panning and keyboard controls remain native. The focused native test
retains the direct reproduction and verifies pixel equality for the protected
adapter.

<a id="cui-021"></a>

## CUI-021 — Sankey flow clicks are omitted from the return value

Reproduced on Linux at `86d9781` with a two-node, one-positive-flow Sankey.
Clicking its ribbon shows native flow hover but returns minus one.
`cui_sankey_ex` computes both a node and link hit, then returns only the node
index on a click.

The FPAS bridge reads the current native hover result on that same native click;
it adds no hit-testing or rendering. It reports node and link activation,
resolving native filtered link indices to original input positions when zero
flows are hidden. Native tests retain the direct omission and protected result;
managed tests exercise original link indices with software and OpenGL.

<a id="cui-022"></a>

## CUI-022 — Table headers and rows omit the keyboard focus ring

Reproduced on Windows at `86d9781` with a sortable column and one selectable
row. Tab moves native focus first to the header and then to the row. The
software-rendered table is identical before and after each Tab because both
elements register keyboard focus without drawing its indicator.

The FPAS table adapter applies CUI's focus ring to the focused header or row
when CUI marks focus visible. The focused `fpas_cui_collection_regressions`
test compares the initial, header-focused and row-focused frames. Running the
same executable with `upstream` exercises the unprotected calls.

<a id="cui-023"></a>

## CUI-023 — A scrolled virtual list can lose its visible range after shrinking

Reproduced on Windows at `86d9781`. Render 100 rows at 36 points each in a
140-point viewport, scroll to offset 2,000 and shrink the list to one row.
The first frame after the change returns `[53,53)` for the only remaining row.
CUI calculates the range using a scroll offset based on the previous height.

The FPAS adapter seeds the current content height and clamps the native scroll
offset and target before CUI calculates the visible range. The focused
`fpas_cui_collection_regressions` test verifies `[0,1)` on that first frame
and retains the direct reproduction in `upstream` mode.

<a id="cui-024"></a>

## CUI-024 — A closing command palette can still activate a command

Reproduced on Windows at `86d9781` with motion enabled and one command.
Draw the open palette, set its open flag to false, and press Enter on the first
closing frame. Native CUI returns command index zero and consumes Enter while
the exit animation is still visible.

The FPAS palette adapter preserves the animation and protects the keyboard
events while the palette is closing or cannot accept input. The focused
`fpas_cui_collection_regressions` test verifies that the closing palette
returns no selection and leaves Enter available. Its `upstream` mode retains
the direct reproduction.

<a id="cui-025"></a>

## CUI-025 — A scrolled data grid can omit all remaining rows after shrinking

Reproduced on Windows at `86d9781`. Render 100 rows at 40 points each in a
160-point viewport, scroll down, then shrink the grid to one row. The first
native frame reports `[1,1)`. This grid calculates its visible range before
reconciling the old scroll geometry.

The FPAS adapter clamps the native scroll offset and target before CUI computes
the range. The focused `fpas_cui_grid_regressions` executable verifies
`[0,1)` immediately and reproduces the omission with its `upstream` argument.

<a id="cui-026"></a>

## CUI-026 — Adding a filter does not open the new chip's value menu

Reproduced on Windows at `86d9781`. Open Filter and choose Team. CUI adds the
chip, but the value menu is absent until the chip is clicked separately. The
native branch that says it opens the value list only pushes and pops an ID.

The FPAS adapter observes the new active bit and opens the native popup on
the following frame, when the chip exists. The focused
`fpas_cui_grid_regressions` executable verifies that both the chip and its
value menu appear after choosing the field. Its `upstream` mode reproduces
the missing popup.

<a id="cui-027"></a>

## CUI-027 — JSON viewer accepts malformed numbers and string escapes

Reproduced on Windows at `86d9781`. Direct `cui_json_view_ex` calls accept
`1e`, `-`, `01`, `"\q"` and `"\u12x4"` as valid. The numeric scanner accepts
number characters in any sequence, and the string scanner skips the character
after a backslash without validating the escape.

The FPAS adapter checks those token grammars before calling CUI. It uses the
native error view and reports the malformed token's location. The native
`fpas_cui_code_instruments` test checks both modes and accepts valid fractions,
exponents and Unicode escapes.

<a id="cui-028"></a>

## CUI-028 — Dot chart activation before arrow input reports an invalid X coordinate

Reproduced on Windows at `86d9781`. Focus a three-sample dot chart with an X axis
from 6 to 30, leave the pointer outside, and press Enter. CUI reports activation
with index minus one and X zero. The first arrow operation starts from the
middle sample, but the activation path leaves the result at its initializer.
The Instruments demo would insert an event outside its time axis.

The FPAS adapter supplies the center sample for a keyboard activation without a
native readout. `fpas_cui_code_instruments` verifies index one, X 18 and the
middle value in this reproduction.

<a id="cui-029"></a>

## CUI-029 — Tab bar Right skips every remaining tab

Reproduced on Windows at `86d9781`. Render the Motion tab bar with four
tabs. Focus Home using Tab, then press Right once. CUI selects Profile and
focuses it. Each subsequent item in the render loop sees the same key press
after the previous item has handed it focus.

The FPAS adapter resolves the arrow target once and presents the native bar
with that selection. The focused `fpas_cui_motion` test checks Home → Search
→ Home and verifies focus at each step. Its `upstream` mode reproduces the skip.

<a id="cui-030"></a>

## CUI-030 — Rolling number silently drops digits and affixes

Reproduced on Windows at `86d9781`. Render `1234567890123456789012345` or
`1 milliseconds per request` with `cui_number_roll`. The 25th digit and
the final letters of the suffix disappear. The renderer measures the full
text, while its fixed buffers keep only 24 rolling slots and 23 bytes
per affix.

FPAS draws complete native tabular text when a value exceeds these buffers.
Values that fit retain native digit animation. `fpas_cui_motion` compares the
rendered glyph geometry with native text for both reproductions.

<a id="cui-031"></a>

## CUI-031 — Gradient slider snapping pulls the value away from the endpoint

Reproduced on Windows at `86d9781`. Focus a gradient slider with range 0–1
and step 0.3. End returns 0.9. With the same range, passing a disabled slider
the value 0.85 also returns 0.9 and reports a change. Native snapping runs
even when the input path has no movement or explicitly requests an endpoint.

The FPAS adapter applies the native step to pointer and arrow movement.
It preserves passive model values and lets Home and End reach the endpoints.
`fpas_cui_creative` checks End, the following idle frame, subsequent arrow
input and a disabled value. Its `upstream` mode reproduces the mismatch.

<a id="cui-032"></a>

## CUI-032 — Read-only schedule items can ignore activation

Reproduced on Windows at `86d9781`. Set `read_only` on a week calendar or
Gantt chart, focus an item with Tab and press Enter. The function returns
minus one. Both builders guard keyboard activation together with keyboard
editing. The week calendar also skips the pointer release that would activate
the appointment and clear the pending press.

The FPAS adapters report keyboard activation of a focused, visible item while
keeping native editing disabled. The week adapter also handles the release
and distinguishes a click from a drag. `fpas_cui_planning` verifies activation
and confirms that an arrow press leaves the event or task unchanged. Its
`upstream` mode reproduces the missing activations.

<a id="cui-033"></a>

## CUI-033 — Typing indicator animates with reduced motion

Reproduced on Windows at `86d9781`. Enable reduced motion, then build a
`cui_typing_indicator` over successive frames. Its dots still advance and
`cui_needs_redraw` stays true once layout has settled.

The FPAS adapter keeps the native dots at their initial phase and preserves
the redraw state from widgets already built in the frame.
`fpas_cui_social upstream` reproduces the continuous redraw request.

<a id="cui-034"></a>

## CUI-034 — Running task glyphs animate with reduced motion

Reproduced on Windows at `86d9781`. Enable reduced motion, then build a
`cui_agent_plan` with a step in `CUI_TASK_RUNNING`. The glyph keeps rotating
and `cui_needs_redraw` stays true after the layout settles. The same native
glyph is used by running tool calls.

The FPAS adapters keep CUI's glyph at its initial phase and preserve animation
requests from earlier widgets. `fpas_cui_voice_ai upstream` reproduces the
continuous redraw. The adapter regression verifies that redraw stops.

<a id="cui-035"></a>

## CUI-035 — Running jobs and tests animate with reduced motion

Reproduced on Windows at `86d9781`. Enable reduced motion and render a
`cui_pipeline` job or a `cui_test_explorer` item in `CUI_JOB_RUNNING`.
The arc still uses the animation clock and keeps `cui_needs_redraw` true.
The FPAS adapters paint the same arc at its initial phase and preserve any
redraw request made by another control. `fpas_cui_developer_tools upstream`
reproduces the native redraw request. The regular test verifies that it stops.

<a id="cui-036"></a>

## CUI-036 — Trace measurements do not set view_changed

Reproduced on Windows at `86d9781`. Shift-drag in the plot of
`cui_trace_view` without changing the time window or vertical scroll.
CUI writes the new measurement to `mark_start` and `mark_end` while leaving
`view_changed` false. An application that applies changes only when the
flag is set can lose the measurement when it redraws from its model.
The FPAS adapter also compares the measurement endpoints.
`fpas_cui_developer_tools measurement` reproduces the missing native
notification. The regular test verifies the FPAS proposal.

<a id="cui-037"></a>

## CUI-037 — Disabled node editor accepts wheel zoom

Reproduced on Windows at `86d9781`. Build a node editor under
`cui_push_disabled(ui, true)`, move the pointer into the canvas and scroll.
The native view's zoom and pan change even though the editor is disabled.
The wheel handler reads the raw scroll target outside CUI's normal interaction
check. The FPAS adapter masks the editor's input while disabled and cancels
any ongoing gesture. `fpas_cui_canvas_bars upstream` reproduces the issue
against native CUI. The same test verifies that FPAS preserves the disabled
view and that enabled wheel zoom and dragging still work.

<a id="cui-038"></a>

## CUI-038 — A disabled veil intercepts clicks

Reproduced on Windows at `86d9781`. Overlay a covered `cui_veil` on an
interactive canvas and disable the veil with `cui_push_disabled(ui, true)`.
Clicks inside the overlap never reach the canvas. CUI registers an
interactive node for the disabled veil and adds a second hit target during
painting.

The FPAS adapter suppresses both hit registrations when the veil is disabled.
The cover still paints. `fpas_cui_display native` reproduces the blocked
click against pinned CUI. The regular test verifies that FPAS forwards the
click to the underlying canvas with the same local coordinates as an
uncovered canvas.

<a id="cui-039"></a>

## CUI-039 — Disabled flame graph and timeline accept wheel zoom

Reproduced on Windows at `86d9781`. Draw a flame graph or timeline inside
`cui_push_disabled(ui, true)`, move the pointer over the plot and scroll up.
Both widgets shrink the visible range. They read the raw scroll target
outside the disabled interaction check.

The FPAS adapters temporarily mask their own wheel target while disabled and
discard any ongoing drag. The timeline also suppresses a middle-button pan
start. `fpas_cui_editing_tools native` reproduces both zooms with direct CUI
calls. The regular test verifies that the original range is preserved by the
adapters.
