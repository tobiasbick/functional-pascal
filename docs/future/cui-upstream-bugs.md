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
