# Native CUI build

Build the private native `fpas_cui` module from the pinned
[CUI release pair](../../../third-party/cui/UPSTREAM.md). The FPAS CMake target
links the platform libraries. `fpas_cui.c` includes the release's `cui.c` and
the adapters that access its private types in the same translation unit.
`bridge.c` handles window options, screenshots and Win32 live-resize callbacks.
[`fpas_cui.h`](fpas_cui.h) declares the native adapters for layout, focus, text
editing, lists, tables, tabs, scrolling, modal dialogs and images. The table
adapter keeps column descriptors alive through the frame. Check the lifetime
of borrowed strings and other inputs for each new adapter.

New widget adapters live in `widgets/` with one implementation and header per
widget. `fpas_cui.c` includes their implementations after the shared helpers
so the adapters can use the private CUI types. The managed bindings are partial
files on `CuiWindow` and `NativeMethods`.

`widgets/button.c` passes the portable button options to CUI and replaces
its label text after creating the native label slot. This keeps literal
`##` text, leading and trailing icons, and loading spinners independent of
the stable identity.

`widgets/copy_button.c` keeps the icon face stable when the glyph switches
from copy to check. The vendored CUI helper changes the key at that point
and loses keyboard focus. The adapter submits text through CUI's clipboard
provider and expires the confirmation on the host's monotonic clock.

The image adapter verifies OpenGL storage after upload. CUI copies pixel data
during the upload and queues the texture handle for frame rendering. Callers
keep that texture alive while CUI can still render the frame.
`fpas_cui_finish_frame_for_destroy` completes deferred paint without presenting
before the binding releases its textures and destroys the window. The software
renderer aborts if its native allocator runs out of memory.

The resize callback draws an open CUI frame on the UI thread. Its caller owns
the callback and user data through detachment or window destruction. Wake targets
post `WM_NULL` on Win32 or an X11 client message through a separate connection.
The managed window serializes posting and release before it destroys the target window.

## Prerequisites

- CMake 3.20 or newer and a C99 compiler.
- Linux: X11 development headers/libraries and the usual C development tools.
- Windows: MSVC with the Windows SDK, or a suitable MinGW toolchain.

The native link, render and editor tests pass for the shared and static builds on
Windows x64 with MSVC and Ubuntu x64 under WSL 2 with GCC. The
[distribution reference](../../../docs/build/distribution.md#native-cui-assets)
describes the prebuilt SDK assets and notices.

## Build and check

Run from the repository root. Keep each RID, configuration and linkage in its
own ignored output directory:

```sh
cmake -S src/Fpas.Ui.Gui.Cui/native \
  -B .temp-data/gui-native/linux-x64/Release/shared \
  -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=ON -DFPAS_CUI_BUILD_TESTS=ON
cmake --build .temp-data/gui-native/linux-x64/Release/shared --parallel
ctest --test-dir .temp-data/gui-native/linux-x64/Release/shared --output-on-failure

cmake -S src/Fpas.Ui.Gui.Cui/native \
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
`fpas::cui`. It initializes the bundled fonts, checks that Enter inserts a line
break in a text area and verifies software-rendered pixels. The editor test checks
multiline keyboard navigation and undo isolation across document replacement.
Both tests run without a display. On Windows, the live-resize test creates a
hidden window and verifies callback registration and detachment.

For Linux ABI inspection, check public symbols in each output:

```sh
nm -D --defined-only .temp-data/gui-native/linux-x64/Release/shared/libfpas_cui.so
nm --defined-only .temp-data/gui-native/linux-x64/Release/static/libfpas_cui.a
```

Shared builds export the `cui_*` API and FPAS bridge while keeping internal symbols hidden.
Static builds omit shared-library import/export definitions and use position
independent code for final NativeAOT linking. MSVC builds the static archive
with `/MT` in both configurations to match the Windows NativeAOT runtime.

The [managed project](../README.md) invokes this build through MSBuild.
Redistributing either library requires its accompanying upstream license notices.
