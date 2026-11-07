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
shortcuts, then global shortcuts. Text and paste bypass shortcuts. The
single-line editor replaces pasted line breaks with spaces.

## Build and inspect

The package contains the TUI library, its terminal adapter and the matching
FPAS runtime support assembly. Use the current local FPAS SDK to build the
source project and the [filter list example](../../examples/tui/filter-list/).
Use the [private-terminal debugger](../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
for interactive inspection at breakpoints.
