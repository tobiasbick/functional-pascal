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

- Upstream: not reported. Fixed on the fork branch `codex/cui-titlebar-testing`
  by `6ddb1ef`; not in the vendored snapshot.

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

- Upstream: not reported. Fixed on the fork branch `codex/cui-titlebar-testing`
  by `b019f9e` and `762fbb9`; not in the vendored snapshot.

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
