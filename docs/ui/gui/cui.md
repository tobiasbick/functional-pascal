# Fpas.Ui.Gui.Cui

Low-level .NET 10 binding for the pinned [CUI dependency](../../../third-party/cui/UPSTREAM.md).
It supports the FPAS GUI host and interop fixtures. Applications use the
[Fpas.Ui.Gui](host.md) MVU/declarative host.
The [widget inventory](../../cui-widget-inventory.md) links to each GUI guide.
The [API coverage reference](../../cui-api-coverage.md) covers host services
and layout primitives.

Managed Veil, Kanban and NodeEditor drawing scopes restrict their native chrome
independently from composed bodies. Callers supply each child control's input
permission explicitly, allowing the host's admitted nested modal to receive
focus while covered chrome stays disabled. A concealed Veil also blocks its
body through CUI's native disabled scope.

## Source layout

The [binding sources](../../../src/Fpas.Ui.Gui.Cui) contain the managed and native adapters.

`Widgets/<Topic>` and `Interop/<Topic>` group the managed adapters and
marshalling code. Widget data uses `Fpas.Ui.Gui.Cui.Widgets.<Topic>`.
The `CuiWindow` and `NativeMethods` partial types each share one namespace
across their topic files. The native C adapters in `native/widgets/<topic>`
use the same topics and expose the `fpas_cui_` symbols.

## Window ownership and frames

`CuiWindow` owns a native context and window on its creating UI thread. One
OpenGL CUI window can own a thread's OpenGL context. Opening another window in
Auto mode selects software, and requesting a second OpenGL window explicitly
fails. Creation accepts minimum dimensions, fixed size and Linux application
identity. [Window placement](app-chrome/window.md) describes restoring saved
dimensions and position. On Windows, an FPAS executable with `ApplicationIcon`
uses its embedded icon for the window caption and Alt+Tab.

`NextFrame` pumps events, presents an open view if needed and starts the next
frame. Close layout scopes in reverse order before ending a frame.
The matching end operation for a box, row, column or scroll container
restores that container as the last widget for sizing, styling and
`TrackLastControl`. FPAS checks the native nesting limit before opening a box
and verifies the native identity before closing it.
`ViewportSize` reads the CUI context's drawable dimensions in logical points.
It reports the initial context default until the first `NextFrame`, then
the dimensions captured at the start of the most recent CUI frame. `Size`
reads the platform's current logical window dimensions.
`Present` completes the current view and exposes finalized control rectangles.
`CapturePng` writes a presented view to an existing directory. After an earlier
presentation, it can also complete a newly constructed view. It releases that
frame's callbacks and surfaces deferred painting failures.

`TryWake` can post from workers with a Win32 `WM_NULL` message or through a
separate X11 connection. The managed window serializes wake posting and target
release before destroying the window. `DeferClose`, `TakeCloseRequest` and
`WakeWhenCompleted` support close decisions and asynchronous cleanup.
`SetTitle`, `Resize` and `RequestFrame` run on the owner thread.
`RequestFrame(0)` calls `cui_request_redraw` for the next frame. Pass a finite
positive delay in seconds to call `cui_request_wakeup`. CUI uses the earliest
pending deadline and can start a frame sooner for input or animation. The next
frame clears the deadline. Call `RequestFrame` from each frame for a recurring
wakeup in an owner-managed window loop. From an MVU `Update` function, return a
[frame request](host.md#invalidation-and-inspection) for a single repaint.

`SetResizeRenderer` roots an owner-thread cached-view callback until
detachment. Failures surface after the native pump returns. The callback draws
an open CUI frame. The bridge completes deferred paint without presenting when
it needs to finish an outstanding frame before destroying the window. See
[drawing callbacks](canvas-and-bars/painting.md) for the lifetime of
`CuiCanvasPainter` and the rules for avoiding window reentry.

`CodeEditorGoTo`, `CodeEditorSelect` and `CodeEditorFind` queue one-view
requests between frames. Each editor ID keeps the most recent request, and
live-resize rendering leaves it queued. The editor translates UTF-16
scalar-boundary offsets to UTF-8 and back. CUI copies visible marks for
deferred painting. See [editor commands](host.md#code-editor-commands).

## Layout and control inspection

The host starts each view with `BeginHostView` and supplies shared sizing and
layout constraints. Text, headings, separators, styles and scroll regions use
native CUI nodes. Dockspace, dock-panel, menu-bar and status-bar
scopes require callers to balance native begin and end operations within the
same frame. Managed layout-stack checks cover split views and other managed layout scopes.

Call `DisabledScope(Disabled, Draw)` with a synchronous drawing callback.
A true flag disables its subtree. A false flag inherits the surrounding
input state. The scope ends when the callback returns or raises.
Complete presentation, capture or disposal after the callback returns.
[Disabled subtrees](layout/container-options.md#disabled-subtrees)

`TrackLastControl` and composite tracking register nodes for geometry, focus
and hover. `ScrollMetrics` reads the vertical offset and range of a tracked
scroll viewport or scrolling box. `ScrollTo` requests an offset. `ScrollReveal`
scrolls the least that shows a control tracked in an earlier frame. A wholly
visible control stays in place. Header snapshots and outer layout nodes can be
tracked separately.
`LastRectangle` calls `cui_last_rect` during a frame. It reads the last emitted
widget's bounds from the previous native layout. It returns a zero rectangle
if that widget has no previous layout.
`ReadDiffRows` and `FlameSpanRectangle` read native data between frames.
After discarding a frame, present another frame before querying them.
`ForgetControl` releases retained state and owned popup or feedback state for
disappearing controls. Private state keys stay in the native bridge. Stable
identities are separate from literal labels, and the FPAS host maps native
interaction results to model proposals. See
[layout](layout/README.md), [widget coverage](../../cui-widget-inventory.md#widget-inventory)
and [host inspection](host.md#invalidation-and-inspection).

## Input injection

Pointer movement, mouse buttons, keys, text and horizontal or vertical wheel
input can be injected during an active frame through the managed binding. These
methods forward to CUI, which routes input according to its current layout and
ownership. Tests using injection exercise the CUI path. Operating-system
keyboard and pointer delivery require platform input tests.
`KeyPressed` reads native frame presses, including repeats. `Modifiers` reads
the native modifier mask. CUI can consume a press before the application reads it.

## Touch input

`Touch`, `MoveTouch` and `CancelTouch` feed CUI's raw multi-touch input for up
to `TouchSlots` fingers during a frame. `TouchCount` and `ReadTouch` return the
frame's state; finger minus one reads the touch that landed first, and a lifted
finger keeps its position for that frame. CUI's Windows and Linux desktop
backends do not report touch yet. Read injected touch state during widget
submission. Capture that state in the paint closure if painting uses it.
[CuiWindow.Touch.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Touch.cs) ·
[NativeMethods.Touch.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Hosting/NativeMethods.Touch.cs) ·
[CuiTouchTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiTouchTests.cs)

## Application shortcuts

Call `ReadShortcuts` once after the frame's widgets have processed input.
It uses native exact combinations, popup, modal and text ownership, as well as
observed held-key levels for the repeat policy. CUI's consumed keys are
unavailable to application bindings. See the
[host shortcut contract](host.md#application-shortcuts).

## Desktop services

`OpenFile`, `SaveFile` and `ChooseFolder` call CUI's modal platform choosers
outside active frames and return a Unicode path or null. Linux uses Zenity or
KDialog. On Windows, CUI preserves the process working directory across Open
and Save. Save opens in the suggested directory and returns the destination
without probing file creation. The caller writes the data. When a Windows
host thread uses multithreaded COM, CUI runs the folder chooser on an STA thread.
`CuiDesktop.TryOpen` dispatches a URL or absolute path to the registered
desktop handler, which may finish asynchronously. Clipboard reads and writes
run on the owner thread through CUI. See [file selection](host.md#file-selection)
and [clipboard commands](host.md#clipboard).

## Theme and resources

`SetTheme` and `ReadTheme` transfer the shared semantic palette.
`SetTheme` accepts optional metrics and `ReadThemeMetrics` returns a detached
snapshot. Set `animate` to true to use CUI's transition when appearance or
accent changes. `ReadTheme` returns the active palette, including colors
between the start and target of a transition. `SystemDark` returns null when
CUI cannot read the platform appearance. `FollowSystemTheme` enables native following. The FPAS host
reapplies its resolved application palette after appearance changes. Reduced
motion is controlled independently of palette changes.

`LoadFont`, `SetFontRole`, `AddFontFallback` and the emoji operations keep
native font data alive until window disposal. Role replacement and restoration
run between frames, as does `SetStrings`, which copies the supplied partial
Unicode caption table. See [fonts and native captions](host.md#fonts-and-native-captions).

## Data and marshalling

The managed bridge validates UTF-8 inputs before passing them to CUI. Native
text parameters use zero-terminated strings, so embedded zero characters and
unpaired UTF-16 surrogates are invalid. Count formats allow bounded integer
`%d` slots and literal `%%` for C varargs calls. Widget-specific marshalling
pairs managed sequential records with native input and result structs. The
canvas input hook reads `cui_input_event` in place; the binding compares its
managed copy with the native size and field offsets before installing the hook. Check
each adapter's buffer lifetime when CUI paints after the call returns. User-menu
avatars reuse native texture loading and lifetime. Guided-tour action identities
stay stable across caption changes.

## Source build and verification

Source builds require a C99 compiler, CMake 3.20 or newer and the platform's
development libraries. See the [native build](native-build.md) for prerequisites.
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

Under WSL, run the Linux suite from a clone in the WSL home directory with
WSL's own .NET SDK, compilers and `xvfb-run`. WSL can clear `/tmp` between
sessions. Building under `/mnt/` or through Windows executables makes the
suite take hours.

`FpasCuiCMakeCommand` selects a non-default CMake executable;
`FpasCuiCMakeArguments` supplies local CMake configuration such as X11 paths.
Build outputs stay in `obj/native/<rid>/<configuration>/shared/`. The source
build accepts matching `linux-x64`/`win-x64` hosts using the .NET SDK's portable
host RID, including distribution-specific SDKs such as Arch Linux.
The [distribution SDK](../../build/distribution.md#native-cui-assets)
includes both native library variants and their notices for its RID.

On Windows, run the suite from a desktop session after building the test project.

```powershell
dotnet test tests/Fpas.Ui.Gui.Cui.Tests/Fpas.Ui.Gui.Cui.Tests.csproj -c Release --no-build --no-restore
```

The tests cover encoding limits and Unicode positions without a display.
Native window cases require a display and are skipped when `DISPLAY` is
absent on Linux. Native Linux dialog automation additionally requires Zenity
with xdotool and xclip on `PATH`. The test project sets `CUI_SCALE=1` for its test host and
fixture processes so the expected PNG pixel dimensions are independent of the
desktop scale. The cases exercise shared-library loading, software and OpenGL
rendering, native button input, thread ownership, nested layout, scroll geometry,
idle wakeup, resize capture and capture-failure recovery. The tests inject input
through CUI's public API. OS keyboard and pointer delivery are separate checks.
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
SDK's portable host RID when it is unset. The build reports unsupported RIDs
and packages missing the selected native asset.

An ordinary build and the framework-dependent, apphost and self-contained
publish modes copy the shared library beside the application. The `single-file`
mode bundles it for extraction by the .NET host. Native AOT links the CUI archive
and OS libraries into the executable using `DirectPInvoke` and `NativeLibrary`.
All modes deliver the six notices under `licenses/cui/`. The CLI ownership
manifest and optional CMS signature cover the executable, any loose native
library and the notices.
See [FPAS publishing](../../build/publishing.md) and
[.NET Native AOT interop](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop).

## FPAS integration fixture

The binding and GUI tests are included in `Fpas.slnx`.
[`FpasProjectTests`](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/FpasProjectTests.cs) builds
the internal [`GuiInterop.fpasproj`](../../../tests/Fixtures/gui/GuiInterop.fpasproj)
using a freshly packaged FPAS SDK, an isolated package cache and a normal CLR
project reference to the solution-built binding. It does not rebuild the native
module or change its restore assets while tests use it.

The FPAS probe presents a button through each renderer and captures a PNG. The
harness verifies native library and license delivery and launches from a separate
working directory without library-search overrides.

### Packaged publishing fixture

Set `FPAS_TEST_GUI_AOT=1` to include the publishing case in the GUI test suite.
Use the [Native AOT prerequisites](../../build/publishing.md#modes-and-prerequisites)
and the CUI source-build prerequisites above. The test also uses `dumpbin` on
Windows and `readelf` on Linux to inspect the executable's imports. On Linux,
use a `TMPDIR` path without spaces for the Native AOT linker.

The [publishing test](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/FpasProjectTests.Publishing.cs)
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

## Source files

- [Presented geometry tests](../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiPresentedGeometryTests.cs)

<details>
<summary>Browse 56 source files</summary>

- `src/Fpas.Ui.Gui.Cui/Build/`
  - [Fpas.Ui.Gui.Cui.targets](../../../src/Fpas.Ui.Gui.Cui/Build/Fpas.Ui.Gui.Cui.targets)
  - [Package.targets](../../../src/Fpas.Ui.Gui.Cui/Build/Package.targets)
- `src/Fpas.Ui.Gui.Cui/`
  - [CuiPrimitives.cs](../../../src/Fpas.Ui.Gui.Cui/CuiPrimitives.cs)
  - [CuiWindow.cs](../../../src/Fpas.Ui.Gui.Cui/CuiWindow.cs)
- `src/Fpas.Ui.Gui.Cui/Hosting/`
  - [CuiDesktop.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiDesktop.cs)
  - [CuiShortcut.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiShortcut.cs)
  - [CuiWindow.Clipboard.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Clipboard.cs)
  - [CuiWindow.Commands.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Commands.cs)
  - [CuiWindow.Dispatch.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Dispatch.cs)
  - [CuiWindow.Host.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Host.cs)
  - [CuiWindow.Input.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Input.cs)
  - [CuiWindow.Shortcuts.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Shortcuts.cs)
  - [CuiWindow.Snapshots.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Snapshots.cs)
  - [CuiWindow.Theme.cs](../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.Theme.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/`
  - [CountFormat.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/CountFormat.cs)
  - [NativeMethods.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/NativeMethods.cs)
  - [Utf8.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Utf8.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeDevResult.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeDevResult.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Files/`
  - [NativeMethods.FileDialog.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Files/NativeMethods.FileDialog.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Hosting/`
  - [NativeMethods.Clipboard.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Hosting/NativeMethods.Clipboard.cs)
  - [NativeMethods.Commands.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Hosting/NativeMethods.Commands.cs)
  - [NativeMethods.Host.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Hosting/NativeMethods.Host.cs)
  - [NativeMethods.Shortcuts.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Hosting/NativeMethods.Shortcuts.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Resources/`
  - [NativeMethods.Colors.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Resources/NativeMethods.Colors.cs)
  - [NativeMethods.Fonts.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Resources/NativeMethods.Fonts.cs)
  - [NativeMethods.Strings.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Resources/NativeMethods.Strings.cs)
- `src/Fpas.Ui.Gui.Cui/Layout/`
  - [CuiWindow.Layout.cs](../../../src/Fpas.Ui.Gui.Cui/Layout/CuiWindow.Layout.cs)
  - [CuiWindow.LayoutOptions.cs](../../../src/Fpas.Ui.Gui.Cui/Layout/CuiWindow.LayoutOptions.cs)
- `src/Fpas.Ui.Gui.Cui/Resources/`
  - [CuiColors.cs](../../../src/Fpas.Ui.Gui.Cui/Resources/CuiColors.cs)
  - [CuiFontFace.cs](../../../src/Fpas.Ui.Gui.Cui/Resources/CuiFontFace.cs)
  - [CuiStrings.cs](../../../src/Fpas.Ui.Gui.Cui/Resources/CuiStrings.cs)
  - [CuiWindow.Fonts.cs](../../../src/Fpas.Ui.Gui.Cui/Resources/CuiWindow.Fonts.cs)
  - [CuiWindow.Strings.cs](../../../src/Fpas.Ui.Gui.Cui/Resources/CuiWindow.Strings.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Audio/`
  - [CuiWindow.AudioValidation.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.AudioValidation.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiDevResult.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiDevResult.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Files/`
  - [CuiFileDialogKind.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiFileDialogKind.cs)
  - [CuiWindow.FileDialog.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiWindow.FileDialog.cs)
- `src/Fpas.Ui.Gui.Cui/native/host/`
  - [bridge.c](../../../src/Fpas.Ui.Gui.Cui/native/host/bridge.c)
  - [colors.c](../../../src/Fpas.Ui.Gui.Cui/native/host/colors.c)
  - [control_state.c](../../../src/Fpas.Ui.Gui.Cui/native/host/control_state.c)
  - [controls.c](../../../src/Fpas.Ui.Gui.Cui/native/host/controls.c)
  - [fonts.c](../../../src/Fpas.Ui.Gui.Cui/native/host/fonts.c)
  - [host_theme.c](../../../src/Fpas.Ui.Gui.Cui/native/host/host_theme.c)
  - [host_widgets.c](../../../src/Fpas.Ui.Gui.Cui/native/host/host_widgets.c)
  - [host_widgets.h](../../../src/Fpas.Ui.Gui.Cui/native/host/host_widgets.h)
  - [resources.h](../../../src/Fpas.Ui.Gui.Cui/native/host/resources.h)
  - [shortcuts.c](../../../src/Fpas.Ui.Gui.Cui/native/host/shortcuts.c)
  - [shortcuts.h](../../../src/Fpas.Ui.Gui.Cui/native/host/shortcuts.h)
  - [strings.c](../../../src/Fpas.Ui.Gui.Cui/native/host/strings.c)
  - [wakeup.c](../../../src/Fpas.Ui.Gui.Cui/native/host/wakeup.c)
  - [widget_nodes.c](../../../src/Fpas.Ui.Gui.Cui/native/host/widget_nodes.c)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [live_resize.c](../../../src/Fpas.Ui.Gui.Cui/native/tests/live_resize.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [developer_data.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/developer_data.c)
  - [developer_data.h](../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/developer_data.h)
- `src/Fpas.Ui.Gui.Cui/native/widgets/files/`
  - [file_dialog.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/files/file_dialog.c)
  - [file_dialog.h](../../../src/Fpas.Ui.Gui.Cui/native/widgets/files/file_dialog.h)

</details>
