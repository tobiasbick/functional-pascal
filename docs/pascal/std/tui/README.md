# `Std.Tui`

`Std.Tui` is the experimental source-level Model–Update–View terminal UI facade. Applications
return a fresh immutable `TuiElement` tree from `View`; they do not create, attach, or destroy live
widgets. It supports deterministic headless tests and the interactive Console terminal.

## Quick reference

| Symbol | Purpose |
| --- | --- |
| `TuiPoint.Create(X, Y)` | Zero-based terminal-cell coordinate. |
| `TuiSize.Create(Width, Height)` | Non-negative terminal-cell extent. |
| `TuiRect.Create(X, Y, Width, Height)` | Half-open rectangle from origin and extents. |
| `TuiColor` / `TuiStyle` / `TuiCell` / `TuiPalette` | Cell painting values and semantic roles. |
| `TuiControlId.Create(Value)` | Positive focus and message-source identity. |
| `TuiAction.Create(Value)` | Positive application intent; values may repeat. |
| `TuiElement` | Closed data-carrying element enum. |
| `TuiElementBuilders` | Constructors for current elements. |
| `TuiSizePolicy` / `TuiAlignment` / `TuiMargins` / `TuiLayoutSettings` | Layout value inputs. |
| `TuiMeasure` / `TuiMeasureSpec` / `TuiMeasureResult` | Pure intrinsic measurement. |
| `TuiMsg` | Key, pointer, tick, resize, focus, action, text/check/selection/scroll change, and quit messages. |
| `TuiPointerEvent` / `TuiPointerButton` | Normalized pointer input in zero-based application coordinates. |
| `TuiMenuItem` / `TuiStatusItem` | Flat menu-bar and status-line chrome descriptions. |
| `TuiFocusFirst` / `TuiFocusResolve` / `TuiLayoutFitFor` / `TuiPaintTooSmallOverlay` | Pure focus, fit, and optional too-small notice helpers. |
| `TuiCmd` | `NoCommand` or `Quit`. |
| `TuiCmdOutput.Set(Command)` | Observable command output passed to `Update`. |
| `TuiApplication.OpenForTest(Size)` | Opens a fixed-size headless host. |
| `App.Inject(Msg)` | Queues one framework message. |
| `App.InjectKeyForTest(Key)` | Queues one key for focus/control routing. |
| `App.InjectPointerForTest(Pointer)` | Queues one pointer event for arranged-frame hit testing. |
| `App.InjectResizeForTest(Size)` | Queues a resize that updates host size before `Update`. |
| `App.RunIterations(...)` | Processes a deterministic message budget. |
| `TuiApplication.Run(...)` | Opens the Console terminal and runs until `TuiCmd.Quit`. |
| `App.SurfaceSnapshot()` | Explicitly copies the last painted surface, including cell roles. |
| `TuiWorkingSurface` | Host-owned mutable cell grid used by paint and headless tests. |
| `App.Close()` | Closes the host and clears pending work. |

## Geometry

`TuiPoint`, `TuiSize`, and `TuiRect` are immutable value records. Coordinates are zero-based.
Sizes and rectangle extents must be non-negative. Rectangles are half-open
(`right = x + width`, `bottom = y + height`).

```pascal
var Bounds: TuiRect := TuiRect.FromEdges(2, 3, 10, 8);
var Inside: boolean := Bounds.Contains(TuiPoint.Create(9, 7));
var Content: TuiRect := Bounds.Inset();
```

| Symbol | Purpose |
| --- | --- |
| `TuiPoint.Create(X, Y)` | Creates a point; all integer coordinates are accepted. |
| `TuiSize.Create(Width, Height)` | Creates a size; rejects negative dimensions. |
| `TuiSize.IsEmpty()` | True when width or height is zero. |
| `TuiRect.Create(X, Y, Width, Height)` | Creates a rectangle; rejects negative or overflowing extents. |
| `TuiRect.FromEdges(Left, Top, Right, Bottom)` | Creates a rectangle from exclusive edges. |
| `TuiRect.FromPointSize(Position, Size)` | Creates a rectangle from a point and size. |
| `TuiRect.FromCorners(TopLeft, BottomRight)` | Creates a rectangle from exclusive corners. |
| `Right()` / `Bottom()` | Exclusive edges. |
| `IsEmpty()` | True when width or height is zero. |
| `Contains(Point)` | Half-open containment. |
| `Intersects(Other)` / `Intersect(Other)` | Overlap test and intersection rectangle. |
| `Inset()` | Shrinks by one cell on every side (clamped to empty). |

For a rectangle at `(2, 3)` with size `(8, 5)`, points `(2, 3)` through `(9, 7)` are inside;
`(10, 7)` and `(9, 8)` are outside.

## Cell values

`TuiColor` has distinct constructors for each representation. `FromCrt` accepts `0..15`, while
`FromAnsi256` and every `FromRgb` channel accept `0..255`.

```pascal
var Foreground: TuiColor := TuiColor.FromCrt(14);
var Background: TuiColor := TuiColor.FromRgb(10, 20, 30);
var Style: TuiStyle := TuiStyle.FromColors(Foreground, Background);
var Cell: TuiCell := TuiCell.Create('X', TuiStyleRole.Focused);
```

`TuiStyle.Create` additionally accepts `Bold`, `Dim`, `Underline`, and `Inverse` flags.
`TuiCell.Create` requires exactly one non-zero-width extended grapheme cluster; `Width()` is one or
two terminal columns via `Std.Console.GraphemeWidth`. The cell stores a semantic `TuiStyleRole`;
concrete colors come from palette lookup. Continuation cells remain private surface state and are
not part of the public cell value.

## `TuiPalette`

`TuiPalette.Default()` provides the standard semantic colors. `ForRole` resolves one style and
`WithRole` returns a copy with one replacement, leaving the original palette unchanged.

```pascal
var Palette: TuiPalette := TuiPalette.Default();
var Warning: TuiStyle := Palette.ForRole(TuiStyleRole.Warning);
var Custom: TuiStyle := TuiStyle.FromColors(TuiColor.FromRgb(255, 128, 0), TuiColor.FromCrt(0));
var Updated: TuiPalette := Palette.WithRole(TuiStyleRole.Accent, Custom);
```

## Elements

`TuiElement` is a data-carrying enum. Its implemented variants are:

```pascal
TuiElement.Empty
TuiElement.Label(Text)
TuiElement.Button(Id, Text, Action)
TuiElement.Input(Id, Text, Caret, ChangeAction)
TuiElement.CheckBox(Id, Text, Checked, ChangeAction)
TuiElement.List(Id, Items, Selected, ChangeAction)
TuiElement.Scroll(Id, Offset, ChangeAction, Children)
TuiElement.MenuBar(Items)
TuiElement.StatusLine(Items)
TuiElement.Row(Children, Spacing)
TuiElement.Column(Children, Spacing)
TuiElement.Layout(Settings, Children)
TuiElement.Spacer(Value)
TuiElement.Window(Title, Children)
TuiElement.Dialog(Title, Children)
TuiElement.Desktop(Focused, Children)
```

`MakeRow` / `MakeColumn` use spacing `0`. Prefer `MakeRowSpaced` / `MakeColumnSpaced` when gaps are
needed. `MakeLayout` and `MakeScroll` wrap exactly one child. `MenuBar` uses `TuiMenuItem` values
(flat enabled/disabled actions with optional display shortcuts). `StatusLine` uses `TuiStatusItem`
hints and commands; hint-only lines are not focusable. Interactive variants cannot omit their typed
control or action identities. Validation before every frame additionally rejects non-positive forged
values, duplicate control ids, invalid input carets, invalid list selection, negative scroll offsets,
and focus ids that do not exist in the tree. Repeated action ids are valid.

Controlled messages include `TextChanged`, `CheckChanged`, `SelectionChanged`, and `ScrollChanged`.
`TuiFocusFirst` / `TuiFocusResolve` choose a valid model focus from the active modal or full tree.
`TuiLayoutFitFor` reports terminal-too-small overflow without changing arranged geometry. Applications
may call `TuiPaintTooSmallOverlay` to replace a working surface with the notice; ordinary frames still
clip overflow as before.

## Layout values and arranged frames

Public layout values include `TuiSizePolicy`, `TuiAlignment`, `TuiMargins`, `TuiLayoutSettings`,
`TuiMeasureSpec`, `TuiMeasureResult`, `TuiSpacer`, and `TuiLayoutFit`. `TuiMeasure(Node, Spec)` is a
pure function. Arrange builds an internal host-owned frame (preorder parent index, bounds, clip)
without copying elements into the index. Applications cannot create, retain, or inspect that frame.
Paint reads the matching frame only and does not remeasure. The application host replaces its
previous tree/frame pair only after validate → arrange → paint has succeeded; routing checks that
same previous pair before interpreting input.

`Row` and `Column` reserve spacing, assign child minimum extents, grow in stable child order toward
preferred extents, then distribute remaining cells by `Stretch` (or `Expanding`). A `TuiLayoutSettings`
wrapper applies its margins and aligns its child within the allocated slot; `Leading`, `Center`,
`Trailing`, and `Fill` are resolved independently on each axis. When the terminal is smaller than
the combined minimum, children retain their minimum geometry and the canvas clips the overflow.

## Update and View

The headless driver accepts this callable shape:

```pascal
function Update(
  State: AppModel;
  Msg: TuiMsg;
  Cmd: TuiCmdOutput
): AppModel;

function View(State: AppModel): TuiElement;
```

Set a command explicitly when needed:

```pascal
TuiMsg.QuitRequested:
begin
  Cmd.Set(TuiCmd.Quit);
  return State
end
```

`TuiCmdOutput` is a host-owned output capability because FPAS `mutable` value parameters permit
local reassignment only; they are not caller-visible output parameters. The host resets the output
to `NoCommand` before every `Update`, reads it immediately afterwards, and stops on `Quit` before
calling `View` or painting again.

## Interactive terminal

`TuiApplication.Run(InitialModel, Update, View)` owns one process terminal for its duration. It
uses the same initial render and update ordering as the headless driver, presents every completed
frame through `Std.Console`, and waits up to 16 ms for an event before emitting a `Tick(16)`.
Keyboard, mouse, and positive resize events are normalized before existing Tui routing; paste and
focus events are intentionally ignored in this version. Mouse coordinates become zero-based.

The host acquires `Std.Console` interactive ownership before the first render and releases it when
`Update` returns `TuiCmd.Quit`. `Std.Console` owns raw mode, alternate-screen, input-feature, and
cursor rollback. Use `RunIterations` and `OpenForTest` for non-interactive regression tests.

## Headless frame and routing order

`RunIterations` renders an initial frame before consuming its iteration budget. Every processed
message then follows:

```text
dequeue → Update → command check → View → validate → layout → paint
```

Injected events are FIFO. Pending routed messages are drained before another external input is
read. One message consumes one iteration. Empty routing results do not synthesize a tick. `Cmd` is
reset to `NoCommand` before every `Update`. A `Quit` command stops before another `View`/paint.

Tab moves through the active focusable subtree. Character/editing keys produce controlled
`TextChanged` messages; Enter or Space activates a focused button. Escape produces
`QuitRequested`. Left-button pointer downs hit-test the previous arranged frame (half-open clip):
focus changes are queued before `Action`, and unhandled pointer input remains `TuiMsg.Pointer`.
`InjectResizeForTest` replaces the host surface size before `TuiMsg.Resize` reaches `Update`.
When a dialog is present directly under the desktop, key and pointer targeting is limited to the
last such dialog subtree.

The current painter paints from the arranged-frame index only: deterministic `Row`/`Column`
allocation (minimum → preferred → expanding leftover), full-size windows, centered dialogs, borders,
labels, controlled inputs, and buttons. The working surface is host-owned and stores leading cells,
wide-glyph continuations, and blanks. Painting goes through a private clipped canvas (local
coordinates, nested origins/clips) and does not construct a full-grid snapshot; `SurfaceSnapshot` is
the explicit copying boundary. Its `CellAt` method returns a `TuiCell`, so screen assertions retain
the painted semantic role as well as the glyph. Overwriting either half of a wide glyph clears both
columns.

## Implementation (contributors)

| Concern | Source |
| --- | --- |
| Elements and invariants | `lib/Std/Tui/Elements/` |
| Geometry and measurement | `lib/Std/Tui/Geometry/`, `lib/Std/Tui/Layout/` |
| Cell, style, and palette values | `lib/Std/Tui/Cells/` |
| Working surface, canvas, and paint | `lib/Std/Tui/Rendering/` |
| Message loop, terminal session, event adapter, and renderer | `lib/Std/Tui/Runtime/` (`Routing/`, `TerminalSession`, `ConsoleEvents`, `TerminalRenderer`) |
| Chrome values | `lib/Std/Tui/Chrome/` |
| FPAS regressions | `tests/stdlib/Tui/` |

## See also

- [Standard library](../README.md)
- [Tui implementation plan](../../../future/Tui/README.md)
- [Testing](../testing/README.md)
