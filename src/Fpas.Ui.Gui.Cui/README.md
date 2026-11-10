# Fpas.Ui.Gui.Cui

Low-level .NET 10 binding for the pinned [CUI dependency](../../third-party/cui/UPSTREAM.md).
It supports the FPAS GUI host and interop fixtures. The application-facing
MVU/declarative GUI library is [Fpas.Ui.Gui](../Fpas.Ui.Gui/README.md).

## Available binding

- `CuiWindow` owns a native window/context. Create, use and dispose it on the
  same UI thread. `TryWake` can run on a worker. It posts a platform event to
  unblock an idle `NextFrame`. It returns false once `NextFrame` observes closure
  or the owner disposes the window. Dispose releases the wake target before
  destroying the window and is idempotent.
- A UI thread can own one OpenGL CUI window at a time. A second window with
  `CuiRenderer.Auto` uses software rendering. A second explicit OpenGL request
  throws, so it cannot replace the current context of the first window.
- `NextFrame` presents the preceding view and begins a new one. Close containers
  in reverse opening order before presenting or capturing a frame. The binding
  also provides headings, text, buttons and checkboxes.
- `ListItem` draws the selection state passed by the host and reports whether
  the item was activated. The host chooses the selection for subsequent frames.
  Give items distinct `###id` suffixes when they can have the same label.
- `TabItem` uses the same selection contract for tab buttons in a horizontal row.
  The host supplies each tab's highlight and chooses the view to draw below it.
- `BeginTable` copies the UTF-8 column titles into the native frame. `TableRow`
  starts a row, paints the supplied selection and reports activation. Add text
  with `TableCell` in column order and finish with `EndTable`. CUI fills omitted
  cells with empty cells.
- `BeginScroll` and `EndScroll` wrap a full-width scroll container. Use a fixed
  height in logical units or zero to fill the available height. CUI scrolls
  overflowing content vertically, with horizontal scrolling available as an
  option. To request a vertical offset, call `ScrollTo` before drawing the
  container in a later frame. Use the same ID and ID scope.
- `BeginDialog` takes a host-owned open flag and updates it on dismissal. Draw
  the contents and call `EndDialog` whenever `BeginDialog` returns true. CUI can
  still draw a closing animation after it clears the open flag. Use a `###id`
  suffix to keep the dialog identity stable when the title changes.
- `RequestFocus` takes a stable control ID during an open frame. CUI consumes
  the request when it next draws a matching focusable control. Call it before
  drawing the target. A later request replaces a pending one. Characters an
  editor has handled stay with that editor if focus moves in the same frame.
- `TextInput` provides single-line editing. `TextArea` wraps multiline text and
  uses a positive logical height (96 by default). Enter submits a single-line
  field and inserts a line break in a text area.
- `CuiTextBuffer` owns bounded, zero-terminated UTF-8 text across frames. The
  array is pinned during native editing calls; these widgets copy paint text
  into CUI-owned frame memory. Keep one buffer per stable editor identity and
  access it on the owning UI thread.
- `Selection` stores the anchor and caret as UTF-16 offsets. Set it before
  drawing an editor and read it after the call, including frames where only the
  selection moved. CUI navigates wrapped lines with the arrow keys, Home and End.
  The binding snaps reported code-point positions to grapheme boundaries in the
  direction of movement and uses those positions on the next draw.
- `SetText` replaces the document and moves the caret to the start. The overload
  taking `CuiTextSelection` sets both together. The next draw clears the editor's
  native undo history and scroll offsets. Invalid Unicode, embedded zeros,
  oversized text or an invalid selection leave the current document intact.
- `CreateImage` copies tightly packed RGBA8 pixels into a texture owned by the
  window. `Image` draws it at an explicit size in logical units. Use and dispose
  images on their owning UI thread. Disposal prevents new draws immediately and
  releases the texture after the last frame that references it can no longer be
  replayed. On window disposal, the binding finishes the open frame without
  presenting it, releases its remaining textures and invalidates any image
  objects the caller still holds before destroying the renderer.
- `Size` reports the size observed by the native event loop. `Resize` requests
  a change asynchronously. `CapturePng` requires an earlier presented frame and
  closed layouts, and ends the current render frame even when writing fails.
- Native calls use source-generated interop, explicit C conventions, one-byte
  native booleans and blittable geometry types. Pointers remain internal.
- `Present` closes and presents a completed frame without waiting for more
  input. Track controls with `TrackLastControl` while drawing and use
  `GetControlRectangle` after presentation for finalized geometry. Tracking
  expires when the next frame begins. `CapturePng` can capture that presented
  frame as well as an open completed view. `RequestClose` asks the next native
  frame pump to stop. Buttons accept an enabled flag for native input gating;
  `FillLastWidth` applies horizontal fill to a widget in a vertical layout.
- Host primitives separate literal labels from identity, reconcile shared flex
  constraints after content measurement, and expose focus/hover, controlled
  editors, scrolling, modal scopes and frame-copied canvases. `CuiHostResources`
  owns declarative editor/image lifetimes. These support the single FPAS MVU
  application path rather than defining a second application API.
- `DeferClose` keeps the native pump alive during asynchronous cleanup and
  exposes `CloseRequested`. `WakeWhenCompleted` wakes it when cleanup finishes.
  `SetResizeRenderer` roots a cached-view callback until detachment; exceptions
  are captured across the unmanaged boundary and rethrown once by `NextFrame`.

## Source build and verification

Source builds require a C99 compiler, CMake 3.20 or newer and the platform's
development libraries. See the [native build](native/README.md) for prerequisites.
The managed project invokes CMake incrementally and copies its shared library
and complete license notices to referencing projects' outputs.
Use source project references for managed builds. For Native AOT publication,
use the [binding package](#packaged-binding) so the final executable receives
the archive and linker settings. With a source reference, setting `PublishAot`
only in the consuming project can leave a shared CUI library beside the
executable. `fpas publish --mode aot` reports the package requirement during
the source build.

```sh
dotnet build src/Fpas.Ui.Gui.Cui/Fpas.Ui.Gui.Cui.csproj -c Release -warnaserror
dotnet build tests/Fpas.Ui.Gui.Cui.Tests/Fpas.Ui.Gui.Cui.Tests.csproj -c Release -warnaserror
xvfb-run -a -s '-screen 0 1400x1000x24 -nolisten tcp -noreset' \
  env LIBGL_ALWAYS_SOFTWARE=1 dotnet test tests/Fpas.Ui.Gui.Cui.Tests/Fpas.Ui.Gui.Cui.Tests.csproj \
  -c Release --no-build --no-restore
```

`FpasCuiCMakeCommand` selects a non-default CMake executable;
`FpasCuiCMakeArguments` supplies local CMake configuration such as X11 paths.
Build outputs stay in `obj/native/<rid>/<configuration>/shared/`. The source
build accepts matching `linux-x64`/`win-x64` hosts.
The [distribution SDK](../../docs/build/distribution.md#native-cui-assets)
includes both native library variants and their notices for its RID.

On Windows, run the suite from a desktop session after building the test project.

```powershell
dotnet test tests/Fpas.Ui.Gui.Cui.Tests/Fpas.Ui.Gui.Cui.Tests.csproj -c Release --no-build --no-restore
```

The tests cover encoding limits and Unicode positions without a display.
Native window cases require a display and are skipped only when `DISPLAY` is
absent on Linux. The test project sets `CUI_SCALE=1` for its test host and
fixture processes so the expected PNG pixel dimensions are independent of the
desktop scale. The cases exercise actual shared-library loading,
software/OpenGL rendering, returned events and compacting managed collections,
thread ownership, nested layout, focus transfer, document replacement, grapheme
boundaries after edits and controlled selection in lists, tables and tabs. They
also cover scroll geometry, modal click routing and dismissal, image replacement
and ownership, idle wakeup during shutdown, resize capture to Unicode filenames
and capture-failure recovery. The tests inject input through CUI's public API.
OS keyboard and pointer delivery are separate checks.
The texture lifetime test counts native software-renderer allocations and
releases during image replacement and repeated window teardown. It injects a
recoverable allocation failure through the renderer interface.
Keep Xvfb's `-noreset` option: the suite repeatedly closes its last window and
immediately opens another, which must not race with a display-server reset.

## Packaged binding

The distribution build writes `Fpas.Ui.Gui.Cui` and `Fpas.NET.Sdk` packages at the
same version to its local feed. The binding package contains the managed
assembly, the shared and static CUI libraries for the build's RID, and the
license notices. Use a feed with the package for the application's target RID.

Add a normal package reference in the FPAS or CLR project that uses CUI:

```xml
<PackageReference Include="Fpas.Ui.Gui.Cui" Version="0.1.0-dev" />
```

The package's `buildTransitive` targets reach the final executable through
project and package references. They select `RuntimeIdentifier`, or the .NET
SDK's host RID when it is unset. The build reports unsupported RIDs and packages
missing the selected native asset.

An ordinary build and the framework-dependent, apphost and self-contained
publish modes copy the shared library beside the application. The `single-file`
mode bundles it for extraction by the .NET host. Native AOT links the CUI archive
and OS libraries into the executable using `DirectPInvoke` and `NativeLibrary`.
All modes deliver the six notices under `licenses/cui/`. The CLI ownership
manifest and optional CMS signature cover the executable, any loose native
library and the notices.
See [FPAS publishing](../../docs/build/publishing.md) and
[.NET Native AOT interop](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop).

## FPAS integration fixture

The binding and GUI tests are included in `Fpas.slnx`.
[`FpasProjectTests`](../../tests/Fpas.Ui.Gui.Cui.Tests/FpasProjectTests.cs) builds
the internal [`GuiInterop.fpasproj`](../../tests/Fixtures/gui/GuiInterop.fpasproj)
using a freshly packaged FPAS SDK, an isolated package cache and a normal CLR
project reference to the solution-built binding. It does not rebuild the native
module or change its restore assets while tests use it.

The FPAS program exercises button activation, by-reference checkbox updates,
UTF-8 editing under compacting collections, layout scopes, dialogs and images.
It checks `using` disposal on normal exit and exception unwinding. Each renderer
must produce a PNG with the expected image color. The harness also verifies native
library/license delivery and launches from a separate working directory without
library-search overrides. The fixture and managed window tests pass on Windows
x64 and on Ubuntu x64 under WSL 2 with Xvfb. Both runs exercise the CUI software
and OpenGL renderers. The WSL run uses software OpenGL.

### Packaged publishing fixture

Set `FPAS_TEST_GUI_AOT=1` to include the publishing case in the GUI test suite.
Use the [Native AOT prerequisites](../../docs/build/publishing.md#modes-and-prerequisites)
and the CUI source-build prerequisites above. The test also uses `dumpbin` on
Windows and `readelf` on Linux to inspect the executable's imports. On Linux,
use a `TMPDIR` path without spaces for the Native AOT linker.

The [publishing test](../../tests/Fpas.Ui.Gui.Cui.Tests/FpasProjectTests.Publishing.cs)
builds both distribution packages in a fresh feed and restores the GUI package
through a C# project reference. It publishes all five modes with `fpas publish`
into the same owned output and checks that mode changes remove stale runtime
files. It verifies each signature with a pinned certificate and checks that
verification rejects changes to the loose CUI library and a license notice.

Both renderers run from a separate working directory without library-search
overrides. For single-file and Native AOT, the test copies only the executable
to a clean directory. Single-file must extract the exact packaged shared library.
Native AOT must run without CUI imports or extraction. The suite runs on Windows
x64 and Ubuntu x64 under WSL 2 with Xvfb.
