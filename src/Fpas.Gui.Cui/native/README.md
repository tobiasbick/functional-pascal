# Native CUI build

Build the private native `fpas_cui` module from the repository's pinned
[single header](../../../third-party/cui/UPSTREAM.md). `fpas_cui.c` is the
only CUI implementation translation unit. `bridge.c` adapts window options,
column configuration and screenshot buffers for the managed binding. Project-owned helpers belong here;
upstream source and licenses remain under `third-party/cui/`.

## Prerequisites

- CMake 3.20 or newer and a C99 compiler.
- Linux: X11 development headers/libraries and the usual C development tools.
- Windows: MSVC with the Windows SDK, or a suitable MinGW toolchain.

The CMake definitions cover Linux and Windows. Linux x64 is verified; Windows
execution and packaging remain in the [implementation plan](../../../docs/future/gui/native-integration.md).

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
library. Multi-configuration generators use `--config Release` for building
and `-C Release` for CTest, and can place outputs in a configuration subdirectory.
Keep their shared import library and static archive in separate build directories.

The optional smoke target is an independent C consumer linked through
`fpas::cui`. It initializes the bundled fonts, creates a widget tree and verifies
software-rendered pixels. It needs no X server. Configure each linkage variant
with tests enabled to check both real linking paths. This does not verify native
window creation or OS input.

For Linux ABI inspection, check public symbols in each output:

```sh
nm -D --defined-only .temp-data/gui-native/linux-x64/Release/shared/libfpas_cui.so
nm --defined-only .temp-data/gui-native/linux-x64/Release/static/libfpas_cui.a
```

Shared builds export the `cui_*` API while keeping internal symbols hidden.
Static builds omit shared-library import/export definitions and use position
independent code for final NativeAOT linking. Both use the same source/header.

The [managed project](../README.md) invokes this build through MSBuild. SDK
packaging and native asset distribution are tracked in the plan.
Redistributing either library requires its accompanying upstream license notices.
