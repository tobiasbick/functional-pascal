# Desktop GUI with CUI

Provide a GUI library for FPAS using [CUI](https://github.com/yappologistic/CUI),
initially targeting Windows x64 and Linux x64. Reuse its widgets, layout and
renderers through ordinary .NET APIs; no FPAS syntax changes are intended.

- Vendor the `cui_single.h` release, initially
  [v1.2.0](https://github.com/yappologistic/CUI/releases/tag/v1.2.0), with its
  version and checksum recorded. Keep the CUI license and complete third-party
  notices, including the embedded fonts and icons.
- Add a small native implementation file and a C# binding, tentatively
  `Fpas.Gui.Cui`. The binding owns native handles, UTF-8 conversion, buffers and
  cleanup, exposing APIs that FPAS can consume directly.
- Integrate the native build into the FPAS build through MSBuild/CMake. Build
  the platform's `cui.dll` or `libcui.so` alongside the binding and include them
  in the matching SDK/distribution. We maintain this integration and update the
  pinned upstream version deliberately.
- GUI projects reference the binding. Their normal build and `fpas publish`
  automatically include the matching native library and license notices.
  Single-file publishing embeds/extracts native assets; Native AOT initially
  ships the library beside the executable. Static linking can be considered later.

End users receive prebuilt native assets. Building FPAS from source requires
the native build tools; Linux applications still need the chosen window backend's
system libraries. Start with CUI's existing desktop backends and renderers.

Verify the integration with a small window, button and text input, then test
build/publish delivery on Windows and Linux. CUI's current accessibility,
complex-text input and direct Wayland limitations remain relevant.

## Shared MVU model

Extract a backend-neutral core from the [TUI host](../../src/Fpas.Tui/README.md),
tentatively `Fpas.Mvu`. Share typed model/message/update transitions, the bounded
message queue, background commands, cancellation and stale-result filtering.
Applications can share their model and update logic when those avoid terminal types.

Keep backend-specific hosts and renderers: terminal cells/layout for TUI, and a
declarative widget description translated into CUI calls for GUI. Views using
only the shared widget subset can be reused; applications can also provide
separate views. The flow remains input → message → update → model and commands
→ view → presentation.

The GUI host processes updates and CUI calls on the UI thread. CUI interactions
enqueue messages for processing after the frame; rendering never directly changes
the application model. Animation redraws reuse the current view without repeating
updates or commands. The model owns application state; CUI keeps transient hover
and animation state associated with stable control IDs. Native text buffers belong
to the binding and report edits through messages.

Test shared transitions/effect lifetimes independently and verify GUI event-to-message
routing alongside the existing TUI contracts.

## Shared widgets and TUI subset

Align the existing TUI widgets with a common FPAS widget contract informed by
CUI. The TUI supports a documented subset; the GUI exposes additional graphical
capabilities. Rework the current widget types around this contract, with one
canonical API per shared widget.

- Share text, buttons, checkboxes, text fields, multiline editors, lists, tables,
  tabs, progress indicators, rows, columns, scrolling and dialogs where their
  semantics fit both backends. Align names, parameters, stable IDs, enabled
  states, model-owned values/selections and event-to-message callbacks.
- Share layout concepts such as growth, alignment and spacing, and semantic
  style roles. Define explicit backend-specific fixed-size units: terminal cells
  versus GUI logical units. Equivalent widgets need not have identical geometry
  or visual styling.
- Adapt text editing explicitly: translate CUI's UTF-8 buffers and editing state
  to the shared model and the existing editor's UTF-16 positions. Preserve the
  agreed caret, selection and navigation behavior.
- Keep graphical features such as images, shadows and pixel/vector canvases as
  GUI extensions. Keep `CellCanvas` and cell/glyph styling as terminal extensions.
  The TUI retains its own renderer and headless operation without a native CUI
  dependency.
- Validate backend support before presentation and diagnose unsupported widgets
  or options clearly. Do not silently drop them or substitute unrelated controls.

Test shared widget values, selection and message behavior against both hosts;
keep layout, rendering and backend-specific interaction tests separate.
