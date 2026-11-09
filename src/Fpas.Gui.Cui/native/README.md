# Native CUI build

Build the private native `fpas_cui` module from the repository's pinned
[CUI sources](../../../third-party/cui/UPSTREAM.md). The upstream CMake target
selects the platform backend and compiles the separate C files. FPAS adds
`bridge.c` for window options, screenshot buffers and Win32 live-resize callbacks.
[`fpas_cui.h`](fpas_cui.h) declares the native adapters for layout, focus, lists,
tables, scrolling and modal dialogs. Tabs and multiline editing use the upstream
entry points. The table adapter keeps column descriptors alive through the frame.

The resize callback draws an open CUI frame on the UI thread. Its caller owns
the callback and user data through detachment or window destruction. Wake targets
post `WM_NULL` on Win32 or an X11 client message through a separate connection.
The managed window serializes posting and release before it destroys the target window.

## Prerequisites

- CMake 3.20 or newer and a C99 compiler.
- Linux: X11 development headers/libraries and the usual C development tools.
- Windows: MSVC with the Windows SDK, or a suitable MinGW toolchain.

The native link and render test passes for the shared and static builds on
Windows x64 with MSVC and Ubuntu x64 under WSL 2 with GCC. The
[distribution reference](../../../docs/build/distribution.md#native-cui-assets)
describes the prebuilt SDK assets and notices.

## Build and check

Run from the repository root. Keep each RID, configuration and linkage in its
own ignored output directory:

```sh
cmake -S src/Fpas.Gui.Cui/native \
  -B .temp-data/gui-native/linux-x64/Release/shared \
  -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=ON -DFPAS_CUI_BUILD_TESTS=ON
cmake --build .temp-data/gui-native/linux-x64/Release/shared --parallel
ctest --test-dir .temp-data/gui-native/linux-x64/Release/shared --output-on-failure

cmake -S src/Fpas.Gui.Cui/native \
  -B .temp-data/gui-native/linux-x64/Release/static \
  -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DFPAS_CUI_BUILD_TESTS=ON
cmake --build .temp-data/gui-native/linux-x64/Release/static --parallel
ctest --test-dir .temp-data/gui-native/linux-x64/Release/static --output-on-failure
```

The outputs are `libfpas_cui.so` and `libfpas_cui.a`. On Windows the corresponding
outputs are `fpas_cui.dll` and `fpas_cui.lib`; shared builds also have an import
library. With the Visual Studio generator, select x64 with `-A x64` at configure
time and `--config Release` at build time. Run CTest with `-C Release` and find
the outputs in the build directory's `Release/` subdirectory. Keep the shared
import library and static archive in separate build directories.

The optional smoke target is an independent C consumer linked through
`fpas::cui`. It initializes the bundled fonts, creates a widget tree and verifies
software-rendered pixels. It needs no X server. Configure each linkage variant
with tests enabled to check both linking paths and the control adapters. On
Windows, the live-resize test also creates a hidden window and verifies callback
registration and detachment.

For Linux ABI inspection, check public symbols in each output:

```sh
nm -D --defined-only .temp-data/gui-native/linux-x64/Release/shared/libfpas_cui.so
nm --defined-only .temp-data/gui-native/linux-x64/Release/static/libfpas_cui.a
```

Shared builds export the `cui_*` API and FPAS bridge while keeping internal symbols hidden.
Static builds omit shared-library import/export definitions and use position
independent code for final NativeAOT linking. Both use the same sources.

The [managed project](../README.md) invokes this build through MSBuild.
Redistributing either library requires its accompanying upstream license notices.
