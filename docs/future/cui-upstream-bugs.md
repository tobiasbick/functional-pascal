# CUI upstream bugs

The `CUI-NNN` identifiers are local references used by the FPAS adapters.
Keep the identifiers stable.

<a id="cui-019"></a>

## CUI-019 — Large constant chart ranges produce non-finite geometry

- Upstream: [PR #64](https://github.com/yappologistic/CUI/pull/64) proposes a fix for the same axis-scaling defect. The [maintainer's adoption report](https://github.com/yappologistic/CUI/pull/64#issuecomment-6883945781) excludes the logic and robustness fixes. The native regression still confirms non-finite geometry at `56c9bc2` on Windows and Linux under WSL.

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

<a id="cui-043"></a>

## CUI-043 — Idle X11 OpenGL windows present continuously

- Upstream: [Issue #104](https://github.com/yappologistic/CUI/issues/104) and
  [PR #105](https://github.com/yappologistic/CUI/pull/105), applied on master as
  [`01f9c06`](https://github.com/yappologistic/CUI/commit/01f9c06) and
  [`bcd918b`](https://github.com/yappologistic/CUI/commit/bcd918b). The vendored
  snapshot `56c9bc2` contains the fix. The maintainer verified it under Xvfb;
  the KWin and Mesa DRI3 reproduction below has not been rerun yet.

Reproduced at `71c383f` with the X11 backend and the OpenGL renderer
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

FPAS has no workaround. Remove this entry once the reproduction is idle with
the vendored snapshot.

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

<a id="cui-049"></a>

## CUI-049 — GCC reports a cleared frame allocation as out of bounds

At `56c9bc2`, GCC 15.2 on Ubuntu reports `-Warray-bounds` or
`-Wstringop-overflow` for the `memset` in `cui__em_button` when it compiles
the amalgamation with `-O3 -std=c99 -Wall`. `71c383f` compiles without warnings with the same flags.
`cui__frame_alloc` can now return `NULL` when a size would overflow. The
caller clears the returned pointer without checking it, so GCC sees a clear
of an object of size 0. `cui__frame_alloc` already returns zeroed memory, so
the clear is redundant. The FPAS Release build treats the warning as an error.

The [native CMake configuration](../../src/Fpas.Ui.Gui.Cui/native/CMakeLists.txt)
disables `-Warray-bounds` and `-Wstringop-overflow` for GCC. A push and pop
around the `cui.c` include does not work because `cui.c` disables
`-Wunused-function` for the rest of the file. Remove the options once the
vendored source no longer warns.
