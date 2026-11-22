# Fpas.Ui.Tui

Terminal adapter for the shared [MVU runtime](../mvu/runtime.md) and
[declarative widgets](../widgets.md). It depends on managed terminal
services and does not load CUI or require a graphical display.

## Application lifecycle

`Run` owns the interactive terminal loop. `StartHeadless` provides `Send`,
`Step`, `Snapshot` and `CloseAsync`. `Update` and `View` run on the owner.
Background work posts typed messages through shared MVU commands. `CloseAsync`
stops admission, requests cancellation and returns a task for cleanup.
Await that task before releasing resources used by effects. `Dispose` joins
outstanding cleanup and releases the host connection. Do not reenter owner
operations from callbacks.
The interactive host restores terminal modes on normal exit and failure.

## Views, layout and validation

The [terminal widget guide](README.md) shows every gallery widget with its example. Views use the shared Element tree, themes, row and column sizing, borders, cached subviews and scrolling.

Preflight validates the view before rendering. Logical distances map to cells:
8 units per column and 16 per row. Measure.Cells explicitly requests cell
distances. Content/fill allocation, weighted growth, shrinkage, size bounds and
cross-axis alignment follow the shared CUI layout subset.

Standalone zero-width format characters and combining marks without a visible
base render as spaces. Combining marks, joiners and variation selectors within
visible grapheme clusters remain intact.

The host retains prepared subviews, allocation data and unchanged painted rows.
An equivalent frame keeps its presentation sequence while installing current
input bindings. Resize resolves the new viewport without invalidating unrelated
cached subviews.

## Cell presentation

The host lowers the prepared view to terminal elements with cell sizes and
resolved styles. Arrangement measures and places those elements, producing
paint operations and a scene of control bounds, focus scopes and scroll
regions. The painter produces rows of cells from those operations and applies
focus and hover decoration. The same cells feed the headless snapshot and
interactive output.

The interactive presenter compares cells with the last successful output and
emits terminal escape sequences for changed cells. A size change or write error
triggers a full repaint.

## Input and controlled state

Tab/Shift+Tab traverse enabled controls in the active scope. An explicit Tab
shortcut can move focus between panes. A [radio group](selection/radio-group.md) has one tab stop. Its unmodified arrow keys move focus and propose a selection. Enter and typed
Space activate the focused action. Rows that support Up/Down focus navigation
use unmodified arrows, leaving modified arrows available for application
shortcuts.
The model owns accepted state changes; rendering never calls change mappers.

Pointer input uses cell coordinates. Disabled and covered controls reject input.
Disabling cancels pending pointer capture.

Single-line text fields accept typing and paste when focused. Left and Right
move by grapheme, Home and End move to either end, and Enter submits the model
value. Click to place the caret. Tab moves on.

Sheets dock to any edge. Dialogs center a content-sized panel in the terminal.
Both dim the background, keep focus inside and scroll a long body. Escape or
a backdrop click proposes dismissal. A persistent dialog stays open until an
action closes it. Closing the overlay restores available focus.

The [file picker](files/file-picker.md) uses the same modal routing. Its
directory, row selection and filename are local to each opening. The model
receives the chosen path or dismissal and controls visibility.

## Popup input

Dropdown menus, single and multiple selection fields, the filter bar and breadcrumb overflow use
the same anchored panel. The active panel
receives keyboard and pointer input. Dismissal returns focus to the trigger
when it is still available. A disabled, removed or covered trigger closes its panel.

## Terminal extensions

- Elements.CellStyled and Terminal.Style supply optional RGB overrides.
- `TerminalExtensions.CellGrids.CellGrid` displays an immutable row-major image
  of `Terminal.Cell` values. Create it with `CellGrids.Image.Create(columns,
  rows, cells)`, or pass null while the first image is being computed. Each
  slot occupies one terminal column. Empty, control, multi-grapheme or
  double-width content appears as a space. Omitted colors inherit the grid style.
  The host clips larger images and pads smaller ones.
- A grid reports `Resized(columns, rows)` through its message mapper after
  its first visible layout, whenever its allocated size changes, and when it
  reappears. Coordinates start at (0, 0) at the unclipped top-left cell.
  `Clicked(column, row, button)` supports left and middle clicks.
  `Wheel(column, row, delta)` retains the signed terminal scroll distance
  (negative is up) and consumes the wheel over the grid. An enclosing scroll
  view can clip the image without changing the reported allocation.
- TerminalExtensions.Views.Viewport resolves a view from exact columns/rows.
- TerminalExtensions.Views.Screen and typed Input shortcuts bind application keys.
- [TerminalExtensions.SelectionLists](data-display/selection-list.md) displays
  a scrollable list with separate selection and activation messages.
- Application.Input/Frame support controlled input injection and cell snapshots.

Snapshots expose visible control IDs, roles, values, bounds, focus and hover,
plus cell text and colors. The gallery's headless-control mode supports the
[private terminal debugger](../../pascal/tools/debugging.md#private-terminal-and-tui-control).

## Themes and examples

Use Fpas.Ui.Widgets.Theming.Themes.Light/Dark and the same ThemeFor callback as GUI. Terminal
styles derive from the shared palette, with alpha composited over cell
backgrounds. Include relevant theme values in cached-subview dependencies.
See [shared themes](../widgets.md#shared-themes). The
[TUI gallery](../../../examples/ui/tui/gallery/README.md) applies them to two
rounded panels with a 1:2 width ratio. Its scrollable sidebar opens a preview
with the GUI’s upload and export examples under **Feedback** and the editor status bar under **App chrome**.

The [CUI inventory](../../cui-widget-inventory.md) records the implemented
terminal widgets and their behavior.

## Source files

<details>
<summary>Browse 24 source files</summary>

- `src/Fpas.Processes/Terminal/`
  - [ConsoleTerminal.cs](../../../src/Fpas.Processes/Terminal/ConsoleTerminal.cs)
  - [VtInput.cs](../../../src/Fpas.Processes/Terminal/VtInput.cs)
- `src/Fpas.Ui/TerminalExtensions/`
  - [Input.fpas](../../../src/Fpas.Ui/TerminalExtensions/Input.fpas)
  - [Views.fpas](../../../src/Fpas.Ui/TerminalExtensions/Views.fpas)
- `src/Fpas.Ui.Tui/`
  - [Application.fpas](../../../src/Fpas.Ui.Tui/Application.fpas)
  - [Fpas.Ui.Tui.fpasproj](../../../src/Fpas.Ui.Tui/Fpas.Ui.Tui.fpasproj)
- `src/Fpas.Ui.Tui/Internal/Controls/`
  - [Activation.fpas](../../../src/Fpas.Ui.Tui/Internal/Controls/Activation.fpas)
- `src/Fpas.Ui.Tui/Internal/Hosting/`
  - [GridNotifications.fpas](../../../src/Fpas.Ui.Tui/Internal/Hosting/GridNotifications.fpas)
  - [Interaction.fpas](../../../src/Fpas.Ui.Tui/Internal/Hosting/Interaction.fpas)
  - [InputDispatch.fpas](../../../src/Fpas.Ui.Tui/Internal/Hosting/InputDispatch.fpas)
  - [Session.fpas](../../../src/Fpas.Ui.Tui/Internal/Hosting/Session.fpas)
- `src/Fpas.Ui.Tui/Internal/Layout/`
  - [Arrangement.fpas](../../../src/Fpas.Ui.Tui/Internal/Layout/Arrangement.fpas)
  - [Equivalence.fpas](../../../src/Fpas.Ui.Tui/Internal/Layout/Equivalence.fpas)
  - [Flexible.fpas](../../../src/Fpas.Ui.Tui/Internal/Layout/Flexible.fpas)
  - [Measurement.fpas](../../../src/Fpas.Ui.Tui/Internal/Layout/Measurement.fpas)
  - [Scene.fpas](../../../src/Fpas.Ui.Tui/Internal/Layout/Scene.fpas)
  - [Stacks.fpas](../../../src/Fpas.Ui.Tui/Internal/Layout/Stacks.fpas)
- `src/Fpas.Ui.Tui/Internal/Rendering/`
  - [Decoration.fpas](../../../src/Fpas.Ui.Tui/Internal/Rendering/Decoration.fpas)
  - [Drawing.fpas](../../../src/Fpas.Ui.Tui/Internal/Rendering/Drawing.fpas)
  - [Presentation.fpas](../../../src/Fpas.Ui.Tui/Internal/Rendering/Presentation.fpas)
  - [Themes.fpas](../../../src/Fpas.Ui.Tui/Internal/Rendering/Themes.fpas)
- `src/Fpas.Ui.Tui/Internal/Terminal/`
  - [Inputs.fpas](../../../src/Fpas.Ui.Tui/Internal/Terminal/Inputs.fpas)
- `src/Fpas.Ui.Tui/Internal/Widgets/Overlays/`
  - [AnchoredPopup.fpas](../../../src/Fpas.Ui.Tui/Internal/Widgets/Overlays/AnchoredPopup.fpas)
  - [PopupHost.fpas](../../../src/Fpas.Ui.Tui/Internal/Widgets/Overlays/PopupHost.fpas)
  - [PopupPlacement.fpas](../../../src/Fpas.Ui.Tui/Internal/Widgets/Overlays/PopupPlacement.fpas)
  - [PopupState.fpas](../../../src/Fpas.Ui.Tui/Internal/Widgets/Overlays/PopupState.fpas)
- `src/Fpas.Ui.Tui/Internal/View/`
  - [Lowering.fpas](../../../src/Fpas.Ui.Tui/Internal/View/Lowering.fpas)
  - [Tree.fpas](../../../src/Fpas.Ui.Tui/Internal/View/Tree.fpas)
- `tests/Fixtures/tui/`
  - [CompositionContract.fpas](../../../tests/Fixtures/tui/CompositionContract.fpas)

</details>
