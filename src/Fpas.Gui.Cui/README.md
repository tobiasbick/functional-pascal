# Fpas.Gui.Cui

Low-level .NET 10 binding for the pinned [CUI dependency](../../third-party/cui/UPSTREAM.md).
It supports the FPAS GUI host and interop fixtures. The application-facing
MVU/declarative GUI library is tracked in the [GUI plans](../../docs/future/gui/README.md).

## Available binding

- `CuiWindow` owns a native window/context. Create, use and dispose it on the
  same UI thread. There is no finalizer that destroys graphics resources from
  another thread. Dispose remains idempotent.
- `NextFrame` presents the preceding view and begins a new one. Complete opened
  columns before another frame. The binding exposes column layout, headings,
  text, buttons, checkboxes and single-line editors, with native input injection
  for host/integration tests.
- `CuiTextBuffer` owns bounded, zero-terminated UTF-8 text across frames. The
  array is pinned during native editing calls; these widgets copy paint text
  into CUI-owned frame memory. Synchronize buffer access with its UI thread.
- Buffer conversion methods map UTF-16 document offsets and UTF-8 byte offsets
  at grapheme boundaries. Invalid/truncated Unicode, embedded zeros and oversized
  replacement text are rejected without truncating or overwriting valid text.
- `Size` reports the size observed by the native event loop. `Resize` requests
  a change asynchronously. `CapturePng` requires an earlier presented frame and
  closed columns, and ends the current render frame even when writing fails.
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
selection and SDK packaging remain open.

The tests cover encoding limits and Unicode positions without a display.
Native window cases require a display and are skipped only when `DISPLAY` is
absent on Linux. Under Xvfb they exercise actual shared-library loading,
software/OpenGL rendering, returned events and compacting managed collections,
thread ownership, resize capture and capture-failure recovery. Input is injected
through CUI's public API; OS keyboard/pointer delivery is a separate check.
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
library-search overrides. Both software and OpenGL cases are verified on Linux.

This fixture verifies emitted FPAS CIL and interop; the public application API
remains the planned MVU/declarative host. Native asset packages and final
publishing modes are separate integration work.

Linux x64 is verified. Windows validation, additional controls/resources,
cross-thread wakeup, native callbacks and FPAS SDK/static NativeAOT publishing
integration remain in their owning plans. The library enables AOT compatibility
analysis; it does not yet provide the final consuming application's static link.
