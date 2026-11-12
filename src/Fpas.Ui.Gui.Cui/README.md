# Fpas.Ui.Gui.Cui

Low-level .NET 10 binding for the pinned [CUI dependency](../../third-party/cui/UPSTREAM.md).
It supports the FPAS GUI host and interop fixtures. Applications use the
[Fpas.Ui.Gui](../Fpas.Ui.Gui/README.md) MVU/declarative host.

## Available binding

- CuiWindow owns a native context and window on its creating UI thread.
  TryWake posts a platform event from workers; disposal releases the wake target
  before destroying the window. An additional Auto window on an OpenGL-owning
  thread uses software rendering; a second explicit OpenGL request is rejected.
- NextFrame pumps events and begins a frame. Close layouts in reverse order;
  Present completes the view and exposes finalized control rectangles.
  CapturePng writes the presented view to an existing directory.
- The binding exposes the accepted button family, theme selectors/toggles,
  sidebar and sheet widgets. Stable identities are separate from literal labels.
  Native interaction results become model proposals in the FPAS host.
  Theme actions compose native segments and icon buttons without a separate
  widget engine or palette mutation.
- Text, headings, separators, box sizing, styles and scrolling are composition
  foundations. The host supplies shared layout constraints to CUI.
- Accordion and expandable-card scopes call CUI's native begin/end pairs;
  expandable text retains its native local state and optional expansion link.
  Header snapshots and outer layout nodes are tracked separately.
- Inline alerts, progress bars, spinners and capacity meters call the native
  CUI widgets. CUI owns dismissal, numeric formatting, threshold colors and motion.
- TrackLastControl and composite tracking expose geometry, focus and hover.
  ForgetControl releases retained state and owned popup/feedback state for
  disappearing controls. Private state keys stay in the native bridge.
- SetTheme/ReadTheme transfer the shared semantic palette. Reduced motion is
  controlled independently of palette changes.
- DeferClose, CloseRequested and WakeWhenCompleted support asynchronous cleanup.
  SetResizeRenderer roots an owner-thread cached-view callback until detachment;
  failures are surfaced after the native pump returns.

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

The FPAS probe presents a button through each renderer and captures a PNG. The
harness verifies native library and license delivery and launches from a separate
working directory without library-search overrides.

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
