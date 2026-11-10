# Fpas.Ui.Tui

Terminal host for [Fpas.Ui](../Fpas.Ui/README.md) views and
[Fpas.Ui.Mvu](../Fpas.Ui.Mvu/README.md) transitions and effects. These are ordinary
.NET 10 records, variants and delegates for FPAS, C# and F# callers. Terminal-only
applications require no GUI binding, CUI assets or native GUI build tools.

## Application lifecycle

Pass an initial model, Update and View to `Fpas.Ui.Tui.Application.Run`.
Update returns `Fpas.Ui.Mvu.Application.Transition<TModel,TMsg>`; View returns
`Fpas.Ui.Elements.Element<TMsg>`. Use constructors in `Fpas.Ui.Elements`,
choices in `Fpas.Ui.Choices`, editor values in `Fpas.Ui.Editors` and layout/style
rules in `Fpas.Ui.Layout` and `Fpas.Ui.Styles`.

`StartHeadless` takes the same callbacks plus positive columns and rows.
It publishes the initial frame and processes one admitted entry per Step.
Send returns a receipt or QueueFull; the FIFO holds 64 entries. Wait observes a
receipt without advancing; WaitForWork waits for input or effect results.
Snapshot returns detached rows, styled cells and control descriptors,
including model-owned editor selection.

Use shared StartBackground, CancelBackground, Focus and Exit commands.
Effects and cancellation callbacks run off the host thread. Exit closes
admission and requests cancellation; CloseAsync awaits cleanup and Dispose
joins outstanding cleanup. Presentation failure retains the last complete
frame. Interactive presentation retries an I/O failure once with a full redraw.

## Views, layout and validation

View takes the model alone or `(Model, Fpas.Ui.Layout.Viewport)`.
Width classes are compact below 60 columns, regular below 100 and expanded
otherwise; height classes use 20 and 35 rows. Resize changes the view and
layout in the same turn without an application message.

Portable views use semantic spacing, roles, content/fill extents and allocation
weights under the [shared CUI rules](../Fpas.Ui/README.md#layout-and-style).
Portable fixed distances use `Measure.Logical`: 8 units per column and 16 per
row, rounded upward. Explicit terminal distances use `Measure.Cells`, for example
`new L.Flex(Basis := L.Extent.Fixed(L.Measure.Cells(20)))`.
Semantic spacing resolves to 0 cells for none, 1 for tight/normal, and 2 columns
or 1 row for loose. Cell layout, Unicode widths, clipping, allocation, painted-row and output
reuse remain native to the terminal host.

The host calls `Validation.Prepare` with Target.Terminal on the entire shared
tree before drawing. Graphical extensions, invalid options,
duplicate identities and invalid selections produce a diagnostic naming the
control or path. Cached subviews reuse typed dependencies; only a successfully
presented tree becomes the next cache. Updated callbacks take effect even when
the host can reuse the terminal frame.

Control and subview IDs are globally unique. Collection keys are local;
action-list snapshots use `list-id/item-key`. Avoid collisions between these
generated IDs and standalone controls; collisions are rejected. Selection lists
and tables use null for no selection and reject unknown keys. Tabs use the first
enabled item when the requested key is null or disabled; unknown non-null keys
are errors. Empty/all-disabled tabs show no panel.

## Input and controlled state

Tab and Shift+Tab traverse enabled controls. Enter activates actions and
eligible selected rows. A typed space also activates buttons and checkboxes.
The [shared sidebar](../Fpas.Ui/README.md#sidebar-navigation) uses the same
destination keys and activation messages as the GUI. Tab moves between its
items, and Enter or Space activates the focused item.
Disabled controls receive no focus or input. Arrows skip disabled collection
items; CanActivate=false leaves an item selectable without activating it.
Tabs support Left/Right, Home/End and skip disabled items.

TextInput and Multiline take text, an Editors.EditorPosition, an edit mapper
and a navigation mapper. Caret and anchor are directional UTF-16 grapheme
boundaries. Update owns text and selection: rejecting a proposal keeps the
model values. Single-line paste converts line breaks to spaces; multiline
paste normalizes them to LF. Visual offsets, preferred columns and caret
following belong to the host. Wheel scrolling and resize emit no selection
change. Editors support arrows, Home/End, Page Up/Down, Ctrl+Home/End, Ctrl+A,
Shift selection and pointer caret placement in terminal cell geometry.

ScrollView supports host-owned scrolling or a model-owned ScrollPosition.
Controlled offsets and ScrollChange.Offset/Maximum use Measure.Cells.
Start and End follow the corresponding edge; explicit offsets are clamped.
Arrows, Page Up/Down, Home/End and wheel input propose changes through OnScroll.
Layout changes resolve positions without messages. Tables keep their header
above the scrolling body and clip whole glyphs within cells.

The top modal owns keyboard and pointer input. It tries its initial focus,
then the first enabled control, and restores previous eligible focus on close.
Mvu.Focus applies the last request after layout/modal reconciliation. Missing,
disabled and out-of-scope targets are ignored.

The [shared sheet](../Fpas.Ui/README.md#sheet-and-drawer) docks to any edge,
with a pinned close button and a dimmed backdrop. Scroll its body with the
wheel or Page Up and Page Down. Escape and a click on the backdrop can dismiss it.

## Terminal extensions

| API | Purpose |
| --- | --- |
| Elements.CellStyled, Terminal.Style | Nullable 24-bit RGB overrides; null inherits. |
| Elements.CellCanvas, Terminal.Cell, Terminal.CanvasEvents | Responsive cell drawing, resize, primary/middle click and wheel input. |
| TerminalExtensions.Views.FixedCanvas | Explicit positive cell width and height. |
| TerminalExtensions.Views.Viewport | Validated subtree built from exact terminal columns/rows. |
| TerminalExtensions.Views.Screen/Layer | Floating layers and scoped shortcuts; portable modals use Elements.Overlay. |
| TerminalExtensions.Views.Panel | Framed body with pinned footer and adaptive padding. |
| TerminalExtensions.Input | Named keys, modifiers, pointer types and typed shortcuts. |
| Fpas.Ui.Tui.Status.Line/Hint | One-row message and priority-ordered complete hints. |
| Fpas.Ui.Tui.Application.Input/Frame | Terminal input injection and cell snapshots. |

Responsive canvas OnResize receives the visible size before drawing;
disappearance reports (0,0). Keep size updates stable so layout converges.
Draw and pointer mappers use coordinates relative to the visible canvas.
Negative wheel deltas scroll up. Cells inherit unspecified colors; focus and
hover preserve canvas colors. Responsive canvases participate in terminal focus;
fixed canvases participate when they have an input mapper.

Screen and layer bindings use Shortcut or CharacterShortcut. Editors consume
typed text first. Active modal shortcuts suspend base shortcuts. Layer limits
cap centered preferred size to 1–100 percent of available space. Panel reserves
its footer first and reduces padding/border when necessary. Status hints name
the active scope and hide least important complete hints as width shrinks.

## Themes and examples

Use `Fpas.Ui.Themes.Light/Dark`, optionally with a CUI Accent, just as in GUI.
Both hosts accept the same Theme and ThemeFor(Model) callback. The terminal
derives semantic styles and button states from its shared palette; there is no
separate terminal palette API. See [shared themes](../Fpas.Ui/README.md#shared-themes)
for customization and alpha compositing. Cached subviews include any theme
values on which their construction depends.

See the [filter list](../../examples/ui/tui/filter-list/README.md),
[gallery](../../examples/ui/tui/gallery/README.md) and
[Mandelbrot explorer](../../examples/ui/tui/mandelbrot/README.md). Use the
[private-terminal debugger](../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
to inspect frames and send input at breakpoints. Cell snapshots expose text
and colors with nullable foreground/background.
