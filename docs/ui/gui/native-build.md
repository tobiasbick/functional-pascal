# Native CUI build

Build the private native `fpas_cui` module from the pinned
[CUI source pair](../../../third-party/cui/UPSTREAM.md). The FPAS CMake target
links the platform libraries.

## Bridge translation unit

[`fpas_cui.c`](../../../src/Fpas.Ui.Gui.Cui/native/fpas_cui.c) includes the generated `cui.c` and
the adapters that access its private types in the same translation unit.
`host/bridge.c` handles window options, screenshots and Win32 live-resize callbacks.
[`fpas_cui.h`](../../../src/Fpas.Ui.Gui.Cui/native/fpas_cui.h) declares the native adapters for layout, focus, text
buttons, navigation, sheets, scrolling and themes. Check the lifetime
of borrowed strings and other inputs for each adapter.

Reuse native CUI primitives before adding adapters. Group adapters by topic in
`widgets/<topic>/`; `fpas_cui.c` includes them after the shared helpers to contain
private CUI types within the native bridge. Managed bindings are partial files
on `CuiWindow` and `NativeMethods`. Compositions need no additional C entry point.

The [Windows transition test](../../../src/Fpas.Ui.Gui.Cui/native/tests/window_transition.c)
reads the WGL swap interval across native window-state changes. It requires a
Windows desktop with OpenGL swap control. CTest reports a skip when the
driver cannot expose the requested interval.

## Native input adapters

Text areas retain CUI's own paint callback and direction-specific wheel routing.
The code-box adapter forwards CUI's completion event through
`CuiOneTimeCodeResult`; the FPAS host sends change before completion. Number-field
reconciliation only handles external model replacements; CUI owns draft validation
and commit/blur formatting. Tag limits and searchable-select dismissal also remain
native.

Property text rows retain the previous proposal to distinguish ordinary editing
from external model replacements. A changed value or capacity resets native
selection and undo history. The inspector owns these bindings and retires them
with their native input state when a row disappears or the inspector is forgotten;
whole-context destruction releases the binding memory without recursive state removal.

The single-date calendar and date picker expose up to 45 native parts: 42 day
cells and the title/page arrows, or twelve month/year choices and those three
header controls. Their adapters use CUI's stable navigation IDs across views.
Range calendars retain their separate native navigation IDs.

## Action captions and identity

`widgets/actions/button.c` passes the portable button options to CUI and replaces
its label text after creating the native label slot. This keeps literal
`##` text, leading and trailing icons, and loading spinners independent of
the stable identity.

`widgets/actions/confirm_button.c` and `widgets/actions/status_button.c` delegate interaction and
animated faces to CUI. Their shared `widgets/actions/morph_button.c` helper restores
literal captions and measures their full width independently of CUI's label-ID
syntax. Removed controls release the corresponding native animation state.

`widgets/actions/segmented.c` calls CUI's segment control and exposes its group and
segment nodes. The managed theme switcher supplies CUI's sun/moon/monitor
icons. Theme toggle uses `widgets/actions/icon_button.c`, which calls CUI's native
icon-button primitive with an identity independent of its glyph and tooltip.
Both propose appearances without mutating the model-owned palette.

The same segment binding draws general FPAS selectors. Empty icon entries map
to native null glyphs. Captions containing `##` use native text slots and content
measurement to keep literal text independent of CUI's label-ID syntax. The
native segment test compares adapter and direct-CUI pixels for every size,
with and without icons and full-width allocation.

`widgets/actions/copy_button.c` delegates drawing, clipboard submission and confirmation
timing to CUI's native copy button. The adapter supplies input permissions,
focus requests and confirmation state for the managed host.

`widgets/actions/split_button.c` uses CUI's composite and popup with literal captions
and keyed inspection. CUI handles keyboard selection. The adapter releases the
popup when its control disappears from the view.

## Composite and popup adapters

Accordion and expandable-card scopes use CUI's native begin/end pairs.
Expandable text retains its native local state and optional expansion link.
The managed bridge exposes header nodes separately from each scope's outer
layout node.

`widgets/overlays/menu.c` calls CUI's menu begin/item/label/separator
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

## Feedback adapters

Alerts, progress bars, spinners and capacity meters call CUI directly. CUI
owns dismissal, numeric formatting, threshold colors and motion. The adapter
supplies stable IDs and reports the state needed to construct the next view.

## Native control lifetime

`host/control_state.c` owns native companion-state and popup retirement. A single
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
[distribution reference](../../build/distribution.md#native-cui-assets)
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

## Native regression tests

The optional smoke target is an independent C consumer linked through
`fpas::cui`. It initializes the bundled fonts, verifies software-rendered pixels,
and checks copy, hold and split-button behavior. Dedicated tests cover confirm,
status, segments, icons, menus, disclosures and feedback, including widget pixel
comparisons against direct CUI calls. These tests run without a display.
On Windows, the live-resize test creates a
hidden window and verifies callback registration and detachment.

## Exported symbols

For Linux ABI inspection, check public symbols in each output:

```sh
nm -D --defined-only .temp-data/gui-native/linux-x64/Release/shared/libfpas_cui.so
nm --defined-only .temp-data/gui-native/linux-x64/Release/static/libfpas_cui.a
```

Shared builds export the `cui_*` API and FPAS bridge while keeping internal symbols hidden.
Static builds omit shared-library import/export definitions and use position
independent code for final NativeAOT linking. MSVC builds the static archive
with `/MT` in both configurations to match the Windows NativeAOT runtime.

## MSBuild integration

The [managed project](cui.md) invokes CMake incrementally through MSBuild for a
source build. The build outputs include the shared library and accompanying
notices. The [distribution build](../../build/distribution.md#native-cui-assets)
supplies both native linkages for packaging.
Redistributing either library requires its accompanying upstream license notices.

## Source files

<details>
<summary>Browse 38 source files</summary>

- `src/Fpas.Ui.Gui.Cui/Build/`
  - [Native.targets](../../../src/Fpas.Ui.Gui.Cui/Build/Native.targets)
- `src/Fpas.Ui.Gui.Cui/`
  - [Fpas.Ui.Gui.Cui.csproj](../../../src/Fpas.Ui.Gui.Cui/Fpas.Ui.Gui.Cui.csproj)
- `src/Fpas.Ui.Gui.Cui/Interop/Containers/`
  - [NativeMethods.Disclosure.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Containers/NativeMethods.Disclosure.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.Feedback.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.Feedback.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Overlays/`
  - [NativeMethods.Menu.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Overlays/NativeMethods.Menu.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Containers/`
  - [CuiWindow.Disclosure.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Containers/CuiWindow.Disclosure.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.Feedback.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Feedback.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.Editor.Types.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.Editor.Types.cs)
  - [CuiWindow.Editor.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.Editor.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Overlays/`
  - [CuiWindow.Menu.Types.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Menu.Types.cs)
  - [CuiWindow.Menu.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Menu.cs)
- `src/Fpas.Ui.Gui.Cui/native/`
  - [CMakeLists.txt](../../../src/Fpas.Ui.Gui.Cui/native/CMakeLists.txt)
  - [fpas_cui.c](../../../src/Fpas.Ui.Gui.Cui/native/fpas_cui.c)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [api_extensions.c](../../../src/Fpas.Ui.Gui.Cui/native/tests/api_extensions.c)
  - [disclosure.c](../../../src/Fpas.Ui.Gui.Cui/native/tests/disclosure.c)
  - [feedback.c](../../../src/Fpas.Ui.Gui.Cui/native/tests/feedback.c)
  - [link_render.c](../../../src/Fpas.Ui.Gui.Cui/native/tests/link_render.c)
  - [menu.c](../../../src/Fpas.Ui.Gui.Cui/native/tests/menu.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [morph_button.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/morph_button.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/containers/`
  - [disclosure.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/disclosure.c)
  - [disclosure.h](../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/disclosure.h)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [feedback.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/feedback.c)
  - [feedback.h](../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/feedback.h)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [text_editor.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/text_editor.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/overlays/`
  - [menu.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/menu.c)
  - [menu.h](../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/menu.h)
  - [modal.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/modal.c)
- `tests/Fixtures/gui/`
  - [GuiInterop.fpasproj](../../../tests/Fixtures/gui/GuiInterop.fpasproj)
  - [GuiPackaged.fpasproj](../../../tests/Fixtures/gui/GuiPackaged.fpasproj)
  - [main.fpas](../../../tests/Fixtures/gui/main.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Packaging/`
  - [FpasProjectTests.Publishing.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/FpasProjectTests.Publishing.cs)
  - [FpasProjectTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/FpasProjectTests.cs)
  - [GuiPackageFixture.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/GuiPackageFixture.cs)
  - [GuiPublishingTests.X11.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/GuiPublishingTests.X11.cs)
  - [GuiPublishingTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/GuiPublishingTests.cs)
  - [NativeDependencies.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/NativeDependencies.cs)
  - [X11Input.Properties.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/X11Input.Properties.cs)
  - [X11Input.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/X11Input.cs)

</details>
