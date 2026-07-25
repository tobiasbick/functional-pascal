# `Std.Tui`

`Std.Tui` is the source-level Model-Update-View terminal UI facade. Applications
return a fresh immutable `TuiElement` tree from `View`; they do not create,
attach, or destroy live widgets. The same application contract supports
deterministic headless tests and an interactive Console terminal.

All public symbols require `uses Std.Tui;`. Private `Std.Tui.*` units are
implementation details.

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
| `TuiPoint.Create(X, Y)` | Zero-based terminal-cell coordinate. |
| `TuiSize.Create(Width, Height)` | Non-negative terminal-cell extent. |
| `TuiRect.Create(X, Y, Width, Height)` | Half-open rectangle from origin and extents. |
| `TuiColor` / `TuiStyle` / `TuiCell` / `TuiPalette` | Cell painting values and semantic roles. |
| `TuiControlId.Create(Value)` | Positive focus and message-source identity. |
| `TuiAction.Create(Value)` | Positive application intent; values may repeat. |
| `TuiElement` / `TuiElementBuilders` | Closed element tree and constructors. |
| `TuiElementBuilders.MakeTextArea(...)` | Controlled multiline editor with model-owned text, caret, and offset. |
| `TuiSizePolicy` / `TuiAlignment` / `TuiMargins` | Layout value inputs. |
| `TuiLayoutSettings.WithFixedHeight(Height)` | Copy of layout settings with one fixed total height. |
| `TuiMeasure` / `TuiMeasureSpec` / `TuiMeasureResult` | Pure intrinsic measurement. |
| `TuiMsg` / `TuiPointerEvent` | Normalized application input. |
| `TuiMenuNode` / `TuiMenuState` / `TuiKeyGesture` | Hierarchical controlled menus. |
| `TuiMenuItem` / `TuiStatusItem` | Flat action-bar and status-line descriptions. |
| `TuiCmd` / `TuiCmdOutput` | Commands emitted by `Update`. |
| `TuiApplication.OpenForTest(Size)` | Opens a fixed-size headless host. |
| `App.RunIterations(...)` | Processes a deterministic message budget. |
| `TuiApplication.Run(...)` | Runs the interactive Console terminal host. |
| `TuiApplication.RunWithPalette(...)` | Runs with a caller-defined initial palette. |
| `App.SurfaceSnapshot()` | Copies the last painted surface for assertions. |

## Implementation (contributors)

| Concern | Source |
| --- | --- |
| Elements and invariants | [`Elements/`](../../../../lib/Std/Tui/Elements/) |
| Geometry and measurement | [`Geometry/`](../../../../lib/Std/Tui/Geometry/), [`Layout/`](../../../../lib/Std/Tui/Layout/) |
| Cell, style, and palette values | [`Cells/`](../../../../lib/Std/Tui/Cells/) |
| Working surface, canvas, and paint | [`Rendering/`](../../../../lib/Std/Tui/Rendering/) |
| Text-area text geometry | [`Text/TextArea.fpas`](../../../../lib/Std/Tui/Text/TextArea.fpas) |
| Application host and routing | [`Runtime/`](../../../../lib/Std/Tui/Runtime/) |
| Chrome values | [`Chrome/`](../../../../lib/Std/Tui/Chrome/) |
| FPAS regressions | [`tests/stdlib/tui/`](../../../../tests/stdlib/tui/) |

## See also

- [Standard library](../README.md)
- [Testing](../testing/README.md)
