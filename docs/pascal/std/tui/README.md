# `Std.Ui.Terminal`

`Std.Ui.Terminal` is the source-level Model-Update-View terminal UI facade. Applications
return an immutable `TuiElement` tree from `View`; they do not create,
attach, or destroy live widgets. The same application contract supports
deterministic headless tests and an interactive Console terminal.

All public symbols require `uses Std.Ui.Terminal as Tui;`. Private `Std.Ui.Terminal.*` units are
implementation details.

## Topics

| Topic | Contents |
| --- | --- |
| [Geometry](geometry.md) | `TuiPoint`, `TuiSize`, and `TuiRect`. |
| [Cells](cells.md) | Colors, styles, cells, and semantic palettes. |
| [Elements](elements.md) | Tree variants, identities, validation, and focus. |
| [Menus](menus.md) | Flat parent-linked menus, popups, mnemonics, and shortcuts. |
| [Text area](text-area.md) | Controlled multiline editing, caret movement, scrolling, and painting. |
| [Tabs](tabs.md) | Model-controlled tab bars, placement, input, overflow, and styles. |
| [Layout](layout.md) | Measurement, arrangement, frames, and clipping. |
| [Application](application.md) | Update/View, headless execution, routing, and the terminal host. |
| [Cached subviews](subviews.md) | Explicit typed dependencies and host-owned subtree reuse. |

## Quick reference

| Symbol | Purpose |
| --- | --- |
| `TuiPoint.Create(X, Y)` | Zero-based terminal-cell coordinate. |
| `TuiSize.Create(Width, Height)` | Non-negative terminal-cell extent. |
| `TuiRect.Create(X, Y, Width, Height)` | Half-open rectangle from origin and extents. |
| `TuiColor` / `TuiStyle` / `TuiCell` / `TuiPalette` | Semantic or concrete truecolor cell painting. |
| `TuiCellGrid.Create(Width, Height, Cells)` | Flat row-major terminal-cell content for custom visualizations. |
| `TuiControlId.Create(Value)` | Positive focus and message-source identity. |
| `TuiAction.Create(Value)` | Positive application intent; values may repeat. |
| `TuiElement` / `TuiElementBuilders` | Closed element tree and constructors. |
| `TuiSubview of T` | Cached subtree with an explicit dependency value and comparison. |
| `TuiElementBuilders.MakeTextArea(...)` | Controlled multiline editor with model-owned text, caret, and offset. |
| `MakePanel` / `MakeOverlay` | Ordinary bordered content and fixed centered modal content. |
| `MakeResponsiveOverlay` / `TuiOverlaySizeRule` | Content-independent centered modal sizing. |
| `MakeTabs` | Controlled tab bar and active page on any of four sides. |
| `MakeRule` / `MakeGauge` / `MakeCellGrid` | Dashboard separators, bounded values, and custom cell views. |
| `TuiSizePolicy` / `TuiAlignment` / `TuiMargins` | Layout value inputs. |
| `TuiLayoutSettings.WithFixedHeight(Height)` | Copy of layout settings with one fixed total height. |
| `TuiMeasure` / `TuiMeasureSpec` / `TuiMeasureResult` | Pure intrinsic measurement. |
| `TuiMsg` / `TuiPointerEvent` | Normalized application input. |
| `TuiMenuNode` / `TuiMenuState` / `TuiKeyGesture` | Hierarchical controlled menus. |
| `TuiMenuItem` / `TuiStatusItem` | Flat action-bar and status-line descriptions. |
| `TuiCmd` / `TuiCmdOutput` | Commands emitted by `Update`. |
| `TuiBackgroundWork` | Cancellable host-owned work returning `result of (boolean, string)`. |
| `TuiApplication.OpenForTest(Size)` | Opens a fixed-size headless host. |
| `App.RunIterations(...)` | Processes a deterministic message budget. |
| `App.RunBackgroundIterations(...)` | Processes framework and typed application messages headlessly. |
| `App.InjectBackgroundForTest(...)` | Tries bounded typed message injection without waiting. |
| `App.CloseWithBackground(Inbox)` | Joins owned work before closing the typed inbox. |
| `TuiApplication.Run(...)` | Runs the interactive Console terminal host. |
| `TuiApplication.RunWithPalette(...)` | Runs with a caller-defined initial palette. |
| `TuiApplication.RunWithBackground(...)` | Runs a wakeable host with a bounded typed inbox. |
| `Cmd.StartBackground(...)` | Starts one host-owned operation after `Update`. |
| `Cmd.ReplaceSubscription(...)` / `CancelSubscription(...)` | Replaces or cancels a long-lived source by id. |
| `Cmd.RequestTick(DelayMilliseconds)` | Asks the interactive host for one `TuiMsg.Tick` after a delay. |
| `App.SurfaceSnapshot()` | Copies the last painted surface for assertions. |

## Implementation (contributors)

| Concern | Source |
| --- | --- |
| Elements and invariants | [`Elements/`](../../../../lib/Std/Ui/Terminal/Elements) |
| Geometry and measurement | [`Geometry/`](../../../../lib/Std/Ui/Terminal/Geometry), [`Layout/`](../../../../lib/Std/Ui/Terminal/Layout) |
| Cell, style, and palette values | [`Cells/`](../../../../lib/Std/Ui/Terminal/Cells) |
| Working surface, canvas, and paint | [`Rendering/`](../../../../lib/Std/Ui/Terminal/Rendering) |
| Text-area text geometry | [`Text/TextArea.fpas`](../../../../lib/Std/Ui/Terminal/Text/TextArea.fpas) |
| Application host and routing | [`Runtime/`](../../../../lib/Std/Ui/Terminal/Runtime) |
| Chrome values | [`Chrome/`](../../../../lib/Std/Ui/Terminal/Chrome) |
| FPAS regressions | [`tests/stdlib/tui/`](../../../../tests/stdlib/tui) |

## See also

- [Mandelbrot background-rendering example](../../../../examples/math/mandelbrot/README.md)
- [Notes application](../../../../apps/notes/README.md)
- [Standard library](../README.md)
- [Testing](../testing/README.md)
