# CUI upstream bugs

The `CUI-NNN` identifiers are local references used by the FPAS adapters.
Keep the identifiers stable.

<a id="cui-019"></a>

## CUI-019 — Large constant chart ranges produce non-finite geometry

- Upstream: [PR #64](https://github.com/yappologistic/CUI/pull/64) proposes a fix for the same axis-scaling defect. The [maintainer's adoption report](https://github.com/yappologistic/CUI/pull/64#issuecomment-6883945781) excludes the logic and robustness fixes. The native regression still confirms non-finite geometry at `71c383f` on Windows and Linux under WSL.

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

<a id="cui-033"></a>

## CUI-033 — Planning drag resumes after editing or its target disappears

Reproduced on Windows using direct `cui_gantt` and `cui_week_calendar` calls
with the vendored snapshot at `a5c9ef5`.

Still reproduced at `71c383f` on Windows by removing only the adapter's drag
guards in an isolated copy. All six read-only, disabled-input and removed-item
cases resume the stale drag. The integration suite passes with the guards.

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

<a id="cui-043"></a>

## CUI-043 — Idle X11 OpenGL windows present continuously

- Upstream: [Issue #104](https://github.com/yappologistic/CUI/issues/104) and
  [PR #105](https://github.com/yappologistic/CUI/pull/105). The PR includes
  regression test `6924c7c` and fix `c13264f` (from `6ddb1ef` on the title bar branch).

Reproduced at `71c383f` and later with the X11 backend and the OpenGL renderer
under Mesa DRI3: Debian, KDE KWin 6.7 X11, VMware SVGA3D. An idle window, with
nothing animating and the pointer outside, used about 25% CPU and swapped
buffers at the swap rate, about 110 to 220 times a second. The software
renderer and Win32 OpenGL idle at 0.

Each `glXSwapBuffers` brings Present events over the X connection, and Mesa
consumes them itself. They wake `select` in `cui__plat_wait` without an Xlib
event for the app. `cui_app_frame` still starts a frame after every wake, and
that frame's swap brings the next event. A gdb trace over 4 idle seconds
counted 444 wakeups and 444 swaps with no `XNextEvent`. The fix keeps waiting
until `XPending` reports an event or the timeout expires. The idle cost then
drops to 0 with no lost caret blinks, input or live resizing.

FPAS has no workaround: GUI applications on Linux with OpenGL pay this CPU
cost while idle.

<a id="cui-044"></a>

## CUI-044 — Transitions that start after an idle pause end in one frame

- Upstream: [Issue #106](https://github.com/yappologistic/CUI/issues/106) and
  [PR #107](https://github.com/yappologistic/CUI/pull/107). The PR includes
  regression test `ce8fbae` and fix `bfdae63` (from `b019f9e` and `762fbb9` on the title bar branch).

Reproduced at `71c383f` and later on Windows (OpenGL and software) and on Linux
X11 (software, and OpenGL once CUI-043 is fixed). Rest the pointer for a second,
then move it onto a button: its hover fill switches to the end colour in one
frame instead of fading over about 150 ms. The same applies to springs and other
transitions that begin on the first frame after an idle wait.

`cui__app_begin` passes the whole idle time as `dt`, and `cui_begin_frame`
caps it only at 0.25 s. `cui__tween` and `cui__spring` then complete a
0.14 s transition in that single frame. The fix gives transitions a separate
step: one 60 Hz frame when the previous frame asked for no other and `dt`
exceeds 50 ms, otherwise `dt`. `dt`, `cui_time` and `cui_delta_time` keep the
real time, so timers, tooltip delays and the caret blink are unchanged.
Capping without the 50 ms threshold slowed hosts that draw below 60 Hz without
requesting frames; `cui_test_canvas` caught that in its 30 fps live-cursor
case.

FPAS has no workaround: hover and other transitions jump whenever the pointer
rests before moving onto a control.

<a id="cui-045"></a>

## CUI-045 — Repeated software screenshots omit unchanged tiles

- Upstream: [Issue #101](https://github.com/yappologistic/CUI/issues/101) and
  [PR #102](https://github.com/yappologistic/CUI/pull/102). The PR includes a
  regression test for capturing twice into the same cleared buffer.

At the pinned revision `71c383f`, capture a static frame with
`cui_app_screenshot`, clear the destination buffer and capture the same frame
into it again. On Linux/X11 the second image differs from the first. The
incremental software renderer recognizes the buffer address and skips tiles
whose drawing commands are unchanged. A fresh allocation can reuse that
address, so consecutive gallery captures can contain blank widgets.

The [PNG capture bridge](../../src/Fpas.Ui.Gui.Cui/native/host/bridge.c)
invalidates the previous software buffer address before each screenshot.
The [gallery capture tests](../../apps/gui-doc-captures/tests/test_capture.py)
exercise consecutive screenshots and check the cropped results.

<a id="cui-046"></a>

## CUI-046 — Windows Save can lose the suggested directory

The vendored Windows chooser passes the whole suggestion to
`GetSaveFileNameW` and allows the dialog to probe whether a file can be
created. In a test directory that denies file creation it returned the
test host's working directory instead of the proposed directory. Turning
off the creation probe alone still returned the wrong directory.

The [Windows chooser adapter](../../src/Fpas.Ui.Gui.Cui/native/widgets/files/win32_file_dialog.c)
uses the Common Item Dialog, sets the proposed folder separately and disables
file creation probes. [The regression](../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiFileDialogTests.cs)
accepts the default Unicode destination in a folder that denies new files.

<a id="cui-047"></a>

## CUI-047 — Windows Choose Folder can hang on an MTA thread

The vendored Windows chooser calls `SHBrowseForFolderW` even when requesting
STA initialization fails on a host thread that already uses MTA. The native
folder regression reproduced the hang with the vendored chooser. Calling the
Common Item Dialog's `Show` on the same MTA thread also hung.

The [Windows chooser adapter](../../src/Fpas.Ui.Gui.Cui/native/widgets/files/win32_file_dialog.c)
uses the Common Item Dialog in folder mode on an STA thread. On an MTA owner
thread, the adapter joins the dialog thread while dispatching synchronous
window messages. [The regression](../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiFileDialogTests.cs)
selects a Unicode directory, cancels the next dialog and presents another frame.

<a id="cui-048"></a>

## CUI-048 — Wheel regions stay at their old position after a dialog moves

[Upstream issue #100](https://github.com/yappologistic/CUI/issues/100) reports
the same routing defect with an image viewer inside a carousel.

At the vendored revision `71c383f`, register a wheel region in a canvas in a
draggable dialog. Scroll over it, drag the header 112 logical points right
and scroll at both positions. The software renderer confirms that the
marked region has moved. The canvas receives the wheel before the drag and
continues receiving it at the old position after the drag. It misses the
wheel at the visible position.

`cui__paint_wheel_hit` intersects the region's layout rectangle with a clip
in paint coordinates. Applying the node transformation before clipping
corrects both hit positions in an isolated source copy. The baseline and
the change produced the same results in three runs each under Windows/MSVC
and WSL/GCC. The vendored implementation still needs the upstream fix.
