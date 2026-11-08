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

### Responsive views

`Run` and `StartHeadless` accept a view of type
`function(TModel, App.Viewport): E.Element<TMsg>` as well as a model-only
view. The viewport gives the full terminal size as `Columns` and `Rows`
in cells. It comes from the terminal for `Run` and from the supplied
dimensions for `StartHeadless`, starting with the initial view.

A resize takes effect when the host processes it. The view and layout use
the new dimensions in the same turn. The resize itself sends no app
message. A responsive canvas can still send its own size message when its
allocated area changes. Include the relevant viewport values in the
dependencies of cached subviews that use them.

## Elements

Compose the screen with `Row`, `Column`, `Border`, `Divider`, `ScrollView` and
`Overlay`. Use `Size` for flex basis, minimum, maximum, grow and shrink.
Containers clip overflowing content. Use `ScrollView` for vertical scrolling.
The controls include `Text`, `Button`, `Checkbox`, `TextInput`, `List`,
`SelectionList`, `Table`, `Tabs`, `ProgressBar` and `CellCanvas`.

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
path. `TextInput` and `Paste` go directly to the focused editor. Single-line
inputs replace line breaks with spaces. Multiline editors preserve line breaks.

### Divider

`Divider()` draws a horizontal rule. `Divider(Title)` puts the title near
the left end with a stroke on each side. In a column with stretch alignment,
the divider fills the available width. It takes one row of preferred height.

The title stays on one line. At narrow widths, the divider shows as many
whole glyphs as fit alongside the strokes and spacing, or just the rule.
It uses the border's horizontal stroke and foreground. Tab skips it.

### Status line

`Fpas.Tui.Status.Line(Message, Hints...)` displays a one-row message on the
left and shortcut hints on the right. Set the message from the model. Use
`Status.Hint(Binding, Label)` to format a hint from the same shortcut you
pass to `Screen` or `Overlay`. Use `Status.Hint(Key, Modifiers, Label)` for
keys such as Tab or an overlay's Escape action.

Order hints by priority. The line reserves space for the message first,
then shows the longest prefix of complete hints that fits. It clips an
oversized message to the available cells. Apply a theme style with `Styled`.
The line displays the hints the app supplies. When opening a modal overlay,
pass the overlay's hints in place of the suspended global shortcuts.

### Overlay sizing

Pass `OverlayLimits` to `Overlay` to cap its width and height as
percentages of the available terminal area. Each percentage is in
1–100 and defaults to 100. The host centers the overlay and uses its
preferred size up to those limits. A cap rounds down to whole cells
and leaves at least one cell in each dimension.

Use `OverlayPanel(Body, Footer, Title, Padding)` for a framed panel with
a pinned footer. Give it a `ScrollView` as the body when content can
overflow. The panel reserves rows for the footer first. It reduces padding
and can omit the frame when space is tight. If only the footer fits, it
receives all available rows. The usual clipping rules apply if the footer
itself exceeds the available area.

For keyboard scrolling, use the controlled `ScrollView` overload and
keep its position in the model. Tab moves between the body and footer
controls according to the overlay's existing focus rules.

### Focus and activation

`Input.Focus(Id)` moves focus to an enabled control in the active focus
scope. `Input.Activate(Id)` runs the control's action through the same
path as unmodified Enter on the focused control. Activation itself leaves
focus in place. A click still uses the pointer position, such as when
placing the caret or selecting a list row.

Return `App.Focus<TMsg>(Id)` from `Update` to focus a control in the
resulting view. The host applies the last focus request of the turn after
layout and modal focus reconciliation. It ignores targets that are missing,
disabled or outside the active focus scope.

### Text input placeholder

`TextInput(Id, Value, OnChange, Placeholder)` displays the optional hint
while the field is empty, including while focused. The caret stays at the
start of the input. The hint contributes to the preferred width and clips to
the allocated width. Line breaks in the hint display as spaces.

The control snapshot, selection and `OnChange` use the actual input value.
Set `Theme.PlaceholderColor` to an RGB foreground color, or use `-1` to
inherit the field foreground.
The built-in presets use their muted foreground for hints.

### Multiline editor

`Editors.Multiline(Id, Text, Position, OnEdit, OnNavigate)` edits a
document from the model. Store its `EditorPosition` alongside the text for
each document. Switching documents can reuse the same control ID and
restore the saved caret, selection and scroll position.

`OnEdit` receives the new text and position together. `OnNavigate` receives
only the new position. Apply both in `Update` and mark the document dirty
when applying an edit. Enter inserts a line break. Paste preserves line
breaks and normalizes inserted CRLF, CR and Unicode line separators to LF.

The editor scrolls logical lines vertically and long lines horizontally.
Arrow keys, Home, End and Page Up/Down move the caret. Ctrl+Home and Ctrl+End
jump to the document boundaries. Shift extends the selection, Ctrl+A selects
all, and a click places the caret. Tab follows the host's focus order.
The viewport follows the caret during editing and navigation. The wheel
scrolls independently until the next caret movement. A resize clamps the
displayed viewport without sending an app message.

Caret and anchor are zero-based UTF-16 offsets clamped to grapheme boundaries.
`TopRow` and `LeftColumn` use zero-based logical rows and terminal cells.
Store the returned position as a whole to retain vertical navigation's
preferred column and the wheel's independent scroll position.

### Selection list

`SelectionList(Id, Items, SelectedKey, OnSelect, OnActivate)` gives the list
one focus stop. Set `SelectedKey` from the model and use an empty string for
no selection. Each `SelectionItem` has a unique nonempty key, a label and a
`CanActivate` flag.

A click or an unmodified navigation key selects a row. The list supports Up,
Down, Home, End, Page Up and Page Down. It keeps the new selection
visible. The wheel scrolls the viewport without changing the selection.
Enter or `Input.Activate(Id)` activates the selected row if it is enabled.
An empty list is skipped by Tab. The control snapshot has role `listbox`
and reports the selected key as its value.

### Table

`Table(Id, Columns, Rows, SelectedKey, OnSelect, OnActivate)` uses the same
selection and activation rules as `SelectionList`. Give each row a unique
nonempty key and one cell per column. Rows can disable activation. The
header stays above the scrolling body and remains visible when the body
is empty. The body has one focus stop when it contains rows.

Each `TableColumn` has a heading and a `Flex` rule. For a fixed width, set
`Basis`, `Minimum` and `Maximum` to the same number of cells. Use
`Grow` for columns that can take extra space. The heading supplies the
basis when `Basis` is unspecified. Text clips within its column, and the
right edge clips columns that cannot fit. Resizing recalculates the
widths and keeps the selected row visible.

The snapshot has role `table`. Its bounds cover the body and its value
is the selected key. Clicking a cell selects its row. Enter or
`Input.Activate(Id)` activates the selected row.

### Tabs

`Tabs(Id, Items, ActiveKey, OnSelect, Content)` builds one panel from its
active key. Set the key in the model and update it in response to
`OnSelect`. Give every `TabItem` a unique nonempty key and a label. The
first enabled tab is shown when the requested key is missing or disabled.
An empty or entirely disabled tab set has no active panel.

The bar is one focus stop. Click a label or use Left and Right to change
tabs. Arrow navigation wraps and skips disabled tabs. Home and End move
to the first or last enabled tab. If the labels overflow, the bar keeps
the active tab visible and shows previous and next controls when space
allows.

Selection keeps focus on the bar. Tab enters the active panel. If a model
update removes a focused panel control, the host returns focus to its bar
when it is eligible. A focus command can target a control in the new panel.
The bar's snapshot has role `tablist` and its value is the active key.

### Controlled scroll view

Use `ScrollView(Id, Child, Position, OnScroll, ShowScrollbar)` to keep the
vertical scroll position in the model. Use `ScrollPosition.Offset(Row)` for
a zero-based top row or `ScrollPosition.End` to keep the newest rows in
view as content grows. The layout clamps the displayed offset to the
available rows.

The focused viewport handles Up, Down, Page Up, Page Down, Home and End.
The wheel scrolls the viewport under the pointer. `OnScroll` receives the
requested position, its resolved offset and the maximum offset. Ordinary
movement requests a fixed offset. End requests follow mode, including
when the last row is already visible. Apply the new position in `Update`.
Layout changes resolve the position again without sending a scroll message.
Text in the scrollable content wraps at the available width, with room
reserved for the scrollbar when it appears.

The scrollbar appears when content overflows and can be disabled with
`ShowScrollbar := false`. The viewport takes one focus stop when it can
scroll. Its snapshot has role `scrollview` and reports the resolved offset
as its value. The two-argument `ScrollView` keeps its offset in the host.

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

`Fpas.Tui.Themes` supplies `Light()`, `Dark()`, `Green()`, `Blue()` and
`Amber()`. Amber uses warm highlights on a dark brown background.
`Dark()` uses the original slate palette, also available as `Slate()`.
Each preset returns a `Palette`. Its `Theme` controls borders, field focus,
list selection and progress. Apply the palette's `Panel`, `Heading`, `Muted`,
`Primary`, `Secondary` and `Error` roles with `Styled`. A different explicit
background color fills the element's clipped bounds, including padding.

Set `Button(..., Variant := ButtonVariant.Primary)` for the main action,
`Secondary` for a supporting action, `Active` for the current choice and
`Destructive` for actions that discard data. The app selects the variant
from its view state. Each preset defines normal, focused and
hovered colors for these variants and a shared disabled style. Disabled
styling takes precedence, then focus, then hover.

Set `Theme.Buttons` to a custom `ButtonStyles` record to supply your own
colors. The host resolves these styles from the current theme each frame.
`ButtonVariant.Default` uses the enclosing style and the usual focus and
hover colors. Themes with an empty `Buttons` slot use that behavior for
all variants. Every variant follows the same button activation rules.

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
controls and layout elements in all five themes.
Use the [private-terminal debugger](../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
for interactive inspection at breakpoints.
