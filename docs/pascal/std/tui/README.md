# `Std.Tui`

`Std.Tui` is the source-level Model-Update-View terminal UI facade. Applications
return a fresh immutable `TuiElement` tree from `View`; they do not create,
attach, or destroy live widgets. The same application contract supports
deterministic headless tests and an interactive Console terminal.

Import `Std.Tui as Tui` for public data types and the common measurement, focus,
and message functions. Constructors and operations are ordinary functions in
the following public units. Pass the affected value as an explicit argument.

| Unit and suggested alias | Functions |
| --- | --- |
| `Std.Tui.Geometry as Geometry` | Points, sizes, rectangles and containment. |
| `Std.Tui.Cells as Cells` | Colors, styles, cells and palettes. |
| `Std.Tui.Chrome as Chrome` | Buttons, inputs, menus, status items and cell grids. |
| `Std.Tui.Elements as Elements` | Element-tree constructors. |
| `Std.Tui.Ids as Ids` | Control and action identities. |
| `Std.Tui.Layout as Layout` | Layout settings, constraints, alignment and spacing. |
| `Std.Tui.Rendering as Rendering` | Working surfaces, snapshots and canvases. |
| `Std.Tui.Runtime as Runtime` | Application hosts, commands and pointer input. |

Other `Std.Tui.*` units are library-internal implementation details. Public
factories retain access to private fields in their declaring units.

## Topics

| Topic | Contents |
| --- | --- |
| [Geometry](geometry.md) | `TuiPoint`, `TuiSize`, and `TuiRect`. |
| [Cells](cells.md) | Colors, styles, cells, and semantic palettes. |
| [Elements](elements.md) | Tree variants, identities, validation, and focus. |
| [Menus](menus.md) | Flat parent-linked menus, popups, mnemonics, and shortcuts. |
| [Text area](text-area.md) | Controlled multiline editing, caret movement, scrolling, and painting. |
| [Layout](layout.md) | Measurement, arrangement, frames, and clipping. |
| [Application](application.md) | Update/View, headless execution, routing, and the terminal host. |

## Quick reference

| Symbol | Purpose |
| --- | --- |
| `Geometry.TuiPointCreate(X, Y)` | Zero-based terminal-cell coordinate. |
| `Geometry.TuiSizeCreate(Width, Height)` | Non-negative terminal-cell extent. |
| `Geometry.TuiRectCreate(X, Y, Width, Height)` | Half-open rectangle from origin and extents. |
| `TuiColor` / `TuiStyle` / `TuiCell` / `TuiPalette` | Semantic or concrete truecolor cell painting. |
| `Chrome.TuiCellGridCreate(Width, Height, Cells)` | Flat row-major terminal-cell content for custom visualizations. |
| `Ids.TuiControlIdCreate(Value)` | Positive focus and message-source identity. |
| `Ids.TuiActionCreate(Value)` | Positive application intent; values may repeat. |
| `Tui.TuiElement` / `Elements.TuiElementMakeLabel` | Closed element tree and ordinary constructors. |
| `Elements.TuiElementMakeTextArea(...)` | Controlled multiline editor with model-owned text, caret, and offset. |
| `Elements.TuiElementMakePanel` / `Elements.TuiElementMakeOverlay` | Ordinary bordered content and fixed centered modal content. |
| `Elements.TuiElementMakeRule` / `Elements.TuiElementMakeGauge` / `Elements.TuiElementMakeCellGrid` | Dashboard separators, bounded values, and custom cell views. |
| `TuiSizePolicy` / `TuiAlignment` / `TuiMargins` | Layout value inputs. |
| `Layout.TuiLayoutSettingsWithFixedHeight(Settings, Height)` | Copy of layout settings with one fixed total height. |
| `Tui.TuiMeasure` / `TuiMeasureSpec` / `TuiMeasureResult` | Minimum and preferred element sizes. |
| `TuiMsg` / `TuiPointerEvent` | Normalized application input. |
| `TuiMenuNode` / `TuiMenuState` / `TuiKeyGesture` | Hierarchical controlled menus. |
| `TuiMenuItem` / `TuiStatusItem` | Flat action-bar and status-line descriptions. |
| `TuiCmd` / `TuiCmdOutput` | Commands emitted by `Update`. |
| `TuiBackgroundWork` | Cancellable host-owned work returning `result of (boolean, string)`. |
| `Runtime.TuiApplicationOpenForTest(Size)` | Opens a fixed-size headless host. |
| `Runtime.TuiApplicationRunIterations(App, ...)` | Processes a deterministic message budget. |
| `Runtime.TuiApplicationRunBackgroundIterations(App, ...)` | Processes framework and typed application messages headlessly. |
| `Runtime.TuiApplicationInjectBackgroundForTest(App, ...)` | Tries bounded typed message injection without waiting. |
| `Runtime.TuiApplicationCloseWithBackground(App, Inbox)` | Joins owned work before closing the typed inbox. |
| `Runtime.TuiApplicationRun(...)` | Runs the interactive Console terminal host. |
| `Runtime.TuiApplicationRunWithPalette(...)` | Runs with a caller-defined initial palette. |
| `Runtime.TuiApplicationRunWithBackground(...)` | Runs a wakeable host with a bounded typed inbox. |
| `Runtime.TuiCmdOutputStartBackground(Cmd, ...)` | Starts one host-owned operation after `Update`. |
| `Runtime.TuiCmdOutputReplaceSubscription(Cmd, ...)` / `Runtime.TuiCmdOutputCancelSubscription(Cmd, Id)` | Replaces or cancels a long-lived source by id. |
| `Runtime.TuiCmdOutputRequestTick(Cmd, DelayMilliseconds)` | Asks the interactive host for one `TuiMsg.Tick` after a delay. |
| `Runtime.TuiApplicationSurfaceSnapshot(App)` | Copies the last painted surface for assertions. |

## Implementation (contributors)

| Concern | Source |
| --- | --- |
| Public function facades | [`Cells.fpas`](../../../../lib/Std/Tui/Cells.fpas), [`Chrome.fpas`](../../../../lib/Std/Tui/Chrome.fpas), [`Elements.fpas`](../../../../lib/Std/Tui/Elements.fpas), [`Geometry.fpas`](../../../../lib/Std/Tui/Geometry.fpas), [`Ids.fpas`](../../../../lib/Std/Tui/Ids.fpas), [`Layout.fpas`](../../../../lib/Std/Tui/Layout.fpas), [`Rendering.fpas`](../../../../lib/Std/Tui/Rendering.fpas), [`Runtime.fpas`](../../../../lib/Std/Tui/Runtime.fpas) |
| Elements and invariants | [`Elements/`](../../../../lib/Std/Tui/Elements/) |
| Geometry and measurement | [`Geometry/`](../../../../lib/Std/Tui/Geometry/), [`Layout/`](../../../../lib/Std/Tui/Layout/) |
| Cell, style, and palette values | [`Cells/`](../../../../lib/Std/Tui/Cells/) |
| Working surface, canvas, and paint | [`Rendering/`](../../../../lib/Std/Tui/Rendering/) |
| Text-area text geometry | [`Text/TextArea.fpas`](../../../../lib/Std/Tui/Text/TextArea.fpas) |
| Application host and routing | [`Runtime/`](../../../../lib/Std/Tui/Runtime/) |
| Chrome values | [`Chrome/`](../../../../lib/Std/Tui/Chrome/) |
| FPAS regressions | [`tests/stdlib/tui/`](../../../../tests/stdlib/tui/) |

## See also

- [Mandelbrot background-rendering example](../../../../examples/math/mandelbrot/README.md)
- [Notes application](../../../../apps/notes/README.md)
- [Standard library](../README.md)
- [Testing](../testing/README.md)
