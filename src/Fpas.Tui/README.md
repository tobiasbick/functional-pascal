# Fpas.Tui

Fpas.Tui is a Model–Update–View terminal library for .NET 10. FPAS, C# and F#
applications use the same typed message and command contracts.

Use `Fpas.Tui.Application.Run` with an initial model, an update function and a
view. `StartHeadless` uses the same callbacks and accepts a width and height
in terminal cells. The headless session publishes its initial frame, then
processes one queued entry per step.

## Application and host

`Update` returns a model and typed commands. `StartBackground` runs work with a
cancellation token and a `Post` function that completes on queue admission.
Starting work with the same ID replaces the previous run. The host discards
already queued results from cancelled or replaced runs. `Exit` closes the
session and joins its cancelled work.

The FIFO holds 64 entries. `Send` returns an input ID after admission or
`QueueFull`. A manual `Step` processes one entry, and `Wait` observes the
outcome without advancing the session. A background result gets its own turn.
`Snapshot` returns detached text rows, styled cells and control descriptors.

The host publishes a new frame after successful presentation, including when
only routing changed. A failure retains the last complete frame. The terminal
presenter retries an I/O failure once with a full redraw of the staged cells.

## Elements

Compose the screen with `Row`, `Column`, `Border`, `ScrollView` and
`Overlay`. Use `Size` for flex basis, minimum, maximum, grow and shrink.
Containers clip overflowing content. Use `ScrollView` for vertical scrolling.
The controls include `Text`, `Button`, `Checkbox`, `TextInput`, `List`,
`ProgressBar` and `CellCanvas`.

Control and cached subview IDs must be unique within the current tree. List
item keys are local to the list, and item controls use `list-id/item-key`.
A cached subview uses its typed dependency comparison. Each session retains
only the cache entries from its most recent complete tree. Geometry allocation,
painted rows and terminal cell runs have separate reuse rules.

A modal overlay owns the focus scope and suspends global shortcuts. The host
tries the requested initial control, then the first focusable control. On
close it restores the old focus if available. Tab and Shift+Tab stay within
the scope. Host navigation precedes the focused control's keys, then scoped
shortcuts, then global shortcuts. Use `Shortcut` for named keys and modifiers.
Use `CharacterShortcut` to match one exact printable character. A typed
character goes to the focused text field first. Otherwise it can trigger a
character shortcut in the active scope. `Input.Character` simulates this
path. `TextInput` and `Paste` go directly to the editor, which replaces line
breaks with spaces.

### Responsive cell canvas

Use `CellCanvas(Id, Draw, Events)` to draw into the space assigned by the layout.
In a stack, wrap the canvas or its border in `Size` with `Grow` to fill the
remaining space. The fixed-size canvas overloads are still available.

Provide `CanvasEvents.OnResize` to map the visible column and row counts to a
typed message. The host delivers the initial size and each subsequent size
change through `Update` before drawing the canvas at that size. A previously
visible canvas that disappears receives `(0, 0)`. Keep resize updates stable so
layout can settle within the same turn.

`Draw`, `OnClick`, `OnMiddleClick` and `OnScroll` use coordinates relative to
the visible canvas, starting at zero. A negative wheel delta means up and a
positive delta means down. A modal overlay blocks pointer input to the canvas
behind it. The canvas owns its cell colors, including while focused or hovered.

## Styling

`Fpas.Tui.Themes` supplies `Light()`, `Dark()`, `Green()` and `Blue()`.
`Dark()` uses the original slate palette, also available as `Slate()`.
Each preset returns a `Palette`. Its `Theme` controls borders, field focus,
list selection and progress. Apply the palette's `Panel`, `Heading`, `Muted`,
`Primary`, `Secondary` and `Error` roles with `Styled`. A different explicit
background color fills the element's clipped bounds, including padding.

Applications can construct a `Palette` with their own `Theme`, `WidgetStyles`
and RGB `Style` values. In FPAS, use `with` to change individual roles of a
preset. Pass a fixed `Theme` to `Run` or `StartHeadless` for a constant look.

To switch at runtime, store the selected palette in the application model and
pass a `ThemeFor` function from the model to `Theme` to either entry point.
`Update` can change the palette in response to a user action. The host resolves
it from the new model for the same frame. Read the palette in `View` for styles
applied with `Styled`. A cached subview that embeds those styles must include
its palette or relevant styles among its dependencies.

The application chooses how to load and save theme preferences, including
preset names and custom RGB colors. The [filter list](../../examples/tui/filter-list/)
demonstrates a live switch from its Options overlay.

## Build and inspect

The package contains the TUI library, its terminal adapter and the matching
FPAS runtime support assembly. Use the current local FPAS SDK to build the
source project and the [filter list example](../../examples/tui/filter-list/).
The [widget gallery](../../examples/tui/gallery/) shows the available
controls and layout elements in all four themes.
Use the [private-terminal debugger](../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
for interactive inspection at breakpoints.
