# Native CUI build

Build the private native `fpas_cui` module from the pinned
[CUI source pair](../../../third-party/cui/UPSTREAM.md). The FPAS CMake target
links the platform libraries. `fpas_cui.c` includes the generated `cui.c` and
the adapters that access its private types in the same translation unit.
`bridge.c` handles window options, screenshots and Win32 live-resize callbacks.
[`fpas_cui.h`](fpas_cui.h) declares the native adapters for layout, focus, text
buttons, navigation, sheets, scrolling and themes. Check the lifetime
of borrowed strings and other inputs for each adapter.

Reuse native CUI primitives before adding adapters. Required adapters live in
`widgets/`; `fpas_cui.c` includes them after the shared helpers to contain
private CUI types within the native bridge. Managed bindings are partial files
on `CuiWindow` and `NativeMethods`. Compositions need no additional C entry point.

`widgets/button.c` passes the portable button options to CUI and replaces
its label text after creating the native label slot. This keeps literal
`##` text, leading and trailing icons, and loading spinners independent of
the stable identity.

`widgets/confirm_button.c` and `widgets/status_button.c` delegate interaction and
animated faces to CUI. Their shared `widgets/morph_button.c` helper restores
literal captions and measures their full width independently of CUI's label-ID
syntax. Removed controls release the corresponding native animation state.

`widgets/segmented.c` calls CUI's segment control and exposes its group and
segment nodes. The managed theme switcher supplies CUI's sun/moon/monitor
icons. Theme toggle uses `widgets/icon_button.c`, which calls CUI's native
icon-button primitive with an identity independent of its glyph and tooltip.
Both propose appearances without mutating the model-owned palette.

The same segment binding draws general FPAS selectors. Empty icon entries map
to native null glyphs. Captions containing `##` use native text slots and content
measurement to keep literal text independent of CUI's label-ID syntax. The
native segment test compares adapter and direct-CUI pixels for every size,
with and without icons and full-width allocation.

`widgets/copy_button.c` delegates drawing, clipboard submission and confirmation
timing to CUI's native copy button. The adapter supplies input permissions,
focus requests and confirmation state for the managed host.

`widgets/split_button.c` uses CUI's composite and popup with literal captions
and keyed inspection. CUI handles keyboard selection. The adapter releases the
popup when its control disappears from the view.

`widgets/menu.c` calls CUI's menu begin/item/label/separator
primitives and exposes keyed commands, input permission, inspection and literal
captions. CUI owns rendering, placement and navigation. The native menu test
compares its open-popup pixels with direct CUI calls for text/icon triggers,
automatic/explicit width and above/below placement.

The same adapter uses `cui_context_menu_begin/end` for context menus attached to
the last component. Anonymous target roots get a native identity and hit
registration; CUI owns hit testing, pointer positioning, rendering and input.
Context-popup ownership is separate from the target's widget state. Context
menus use the same command conversion, rows, inspection and retirement paths
as dropdowns. The pixel test also compares a context popup with direct CUI.

`control_state.c` owns native companion-state and popup retirement. A single
forget operation retires a removed control; changing a control's kind retires
its former companion state. Managed code does not encode private state keys.

`fpas_cui_finish_frame_for_destroy` completes deferred paint without presenting
before the binding destroys the window. The software
renderer aborts if its native allocator runs out of memory.

The resize callback draws an open CUI frame on the UI thread. Its caller owns
the callback and user data through detachment or window destruction. Wake targets
post `WM_NULL` on Win32 or an X11 client message through a separate connection.
The managed window serializes posting and release before it destroys the target window.

## Prerequisites

- CMake 3.20 or newer and a C99 compiler.
- Linux: X11 development headers/libraries and the usual C development tools.
- Windows: MSVC with the Windows SDK, or a suitable MinGW toolchain.

The native link, render and widget tests cover the shared and static builds on
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
`fpas::cui`. It initializes the bundled fonts, verifies software-rendered pixels,
and checks copy, hold and split-button behavior. Dedicated tests cover confirm,
status, segments, icons, menus and disclosures, including disclosure pixel
comparisons against direct CUI calls. These tests run without a display.
On Windows, the live-resize test creates a
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
