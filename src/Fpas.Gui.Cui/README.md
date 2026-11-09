# Fpas.Gui.Cui

Low-level .NET 10 binding for the pinned [CUI dependency](../../third-party/cui/UPSTREAM.md).
It supports the FPAS GUI host and interop fixtures. The application-facing
MVU/declarative GUI library is tracked in the [GUI plans](../../docs/future/gui/README.md).

## Available binding

- `CuiWindow` owns a native window/context. Create, use and dispose it on the
  same UI thread. `TryWake` can run on a worker. It posts a platform event to
  unblock an idle `NextFrame`. It returns false once `NextFrame` observes closure
  or the owner disposes the window. Dispose releases the wake target before
  destroying the window and is idempotent.
- `NextFrame` presents the preceding view and begins a new one. Close rows,
  columns and tables in reverse opening order before presenting or capturing a
  frame. The binding also provides headings, text, buttons, checkboxes and
  single-line editors.
- `ListItem` draws the selection state passed by the host and reports whether
  the item was activated. The host chooses the selection for subsequent frames.
  Give items distinct `###id` suffixes when they can have the same label.
- `TabItem` uses the same selection contract for tab buttons in a horizontal row.
  The host supplies each tab's highlight and chooses the view to draw below it.
- `BeginTable` copies the UTF-8 column titles into the native frame. `TableRow`
  starts a row, paints the supplied selection and reports activation. Add text
  with `TableCell` in column order and finish with `EndTable`. CUI fills omitted
  cells with empty cells.
- `RequestFocus` takes a stable control ID during an open frame. CUI consumes
  the request when it next draws a matching focusable control. Call it before
  drawing the target. A later request replaces a pending one. Characters an
  editor has handled stay with that editor if focus moves in the same frame.
- `CuiTextBuffer` owns bounded, zero-terminated UTF-8 text across frames. The
  array is pinned during native editing calls; these widgets copy paint text
  into CUI-owned frame memory. Synchronize buffer access with its UI thread.
- Buffer conversion methods map UTF-16 document offsets and UTF-8 byte offsets
  at grapheme boundaries. Invalid/truncated Unicode, embedded zeros and oversized
  replacement text are rejected without truncating or overwriting valid text.
- `Size` reports the size observed by the native event loop. `Resize` requests
  a change asynchronously. `CapturePng` requires an earlier presented frame and
  closed layouts, and ends the current render frame even when writing fails.
- Native calls use source-generated interop, explicit C conventions, one-byte
  native booleans and blittable geometry types. Pointers remain internal.

## Source build and verification

Source builds require a C99 compiler, CMake 3.20 or newer and the platform's
development libraries. See the [native build](native/README.md) for prerequisites.
The managed project invokes CMake incrementally and copies its shared library
and complete license notices to referencing projects' outputs.

```sh
dotnet build src/Fpas.Gui.Cui/Fpas.Gui.Cui.csproj -c Release -warnaserror
dotnet build tests/Fpas.Gui.Cui.Tests/Fpas.Gui.Cui.Tests.csproj -c Release -warnaserror
xvfb-run -a -s '-screen 0 1400x1000x24 -nolisten tcp -noreset' \
  env LIBGL_ALWAYS_SOFTWARE=1 dotnet test tests/Fpas.Gui.Cui.Tests/Fpas.Gui.Cui.Tests.csproj \
  -c Release --no-build --no-restore
```

`FpasCuiCMakeCommand` selects a non-default CMake executable;
`FpasCuiCMakeArguments` supplies local CMake configuration such as X11 paths.
Build outputs stay in `obj/native/<rid>/<configuration>/shared/`. The source
build accepts matching `linux-x64`/`win-x64` hosts; prebuilt cross-RID asset
selection for consuming projects belongs to [GUI publishing](../../docs/future/gui/fpas-publishing.md).
The [distribution SDK](../../docs/build/distribution.md#native-cui-assets)
includes both native library variants and their notices for its RID.

On Windows, run the suite from a desktop session after building the test project.

```powershell
dotnet test tests/Fpas.Gui.Cui.Tests/Fpas.Gui.Cui.Tests.csproj -c Release --no-build --no-restore
```

The tests cover encoding limits and Unicode positions without a display.
Native window cases require a display and are skipped only when `DISPLAY` is
absent on Linux. The test project sets `CUI_SCALE=1` for its test host and
fixture processes so the expected PNG pixel dimensions are independent of the
desktop scale. The cases exercise actual shared-library loading,
software/OpenGL rendering, returned events and compacting managed collections,
thread ownership, nested layout, focus transfer and controlled selection in
lists, tables and tabs. They also cover idle wakeup during shutdown, resize
capture and capture-failure recovery. Input
is injected through CUI's public API. OS keyboard and pointer delivery are
separate checks.
Keep Xvfb's `-noreset` option: the suite repeatedly closes its last window and
immediately opens another, which must not race with a display-server reset.

## FPAS integration fixture

The binding and GUI tests are included in `Fpas.slnx`.
[`FpasProjectTests`](../../tests/Fpas.Gui.Cui.Tests/FpasProjectTests.cs) builds
the internal [`GuiInterop.fpasproj`](../../tests/Fixtures/gui/GuiInterop.fpasproj)
using a freshly packaged FPAS SDK, an isolated package cache and a normal CLR
project reference to the solution-built binding. It does not rebuild the native
module or change its restore assets while tests use it.

The FPAS program exercises button activation, by-reference checkbox updates,
UTF-8 editing under compacting collections and native geometry. It checks
`using` disposal on normal exit and exception unwinding. Each renderer must
produce a nonuniform PNG of the expected size. The harness also verifies native
library/license delivery and launches from a separate working directory without
library-search overrides. The fixture and managed window tests pass on Windows
x64 and on Ubuntu x64 under WSL 2 with Xvfb. Both runs exercise the CUI software
and OpenGL renderers. The WSL run uses software OpenGL.

This fixture verifies emitted FPAS CIL and interop. The planned MVU/declarative
host owns the public application API. Package-based binding consumption and final
publishing modes are separate integration work.

Additional controls and resources, managed resize callbacks and FPAS publishing
integration remain in their owning plans. The library enables AOT compatibility
analysis.
