# Fpas.Ui

Fpas.Ui defines the declarative view used by the GUI and TUI adapters.
It is an FPAS library with ordinary .NET 10 records, variants and
delegates. It can be used with [Fpas.Ui.Mvu](../Fpas.Ui.Mvu/README.md) and has no
dependency on a host, renderer or CUI.

## Building a view

Use the constructors in `Fpas.Ui.Elements` to build an `Element<TMsg>` from
the model. A view can capture a message directly or use a callback to turn an
input value into a message. The host invokes input callbacks for accepted
interactions and dispatches their messages through MVU. Values supplied by the
model remain authoritative until an update supplies new values.

Keep interactive control IDs, modal IDs and cached subview IDs nonempty,
unique throughout the view, and stable between frames. Treat strings as
case-sensitive. Item and row keys are unique within their own list, table or
tab set. An item key can repeat in another collection. Keep view records,
arrays, callbacks and image data unchanged for as long as the host retains
them. Replace changed data on a subsequent view.

## Capability profiles

`Fpas.Ui.Validation.Prepare` checks the following profiles. The table defines
the contract accepted by the validator. The host references describe rendering
and input support. [GUI adapter](../Fpas.Ui.Gui/README.md),
[TUI adapter](../Fpas.Ui.Tui/README.md).

| Widget or option | Portable | Terminal | Graphical |
| --- | --- | --- | --- |
| Text and titled or plain divider | Yes | Yes | Yes |
| Button variants, sizes, icons, busy state and width | Yes | Yes | Yes |
| Button group with a shared outline | Yes | Yes | Yes |
| Copy button with transient feedback | Yes | Yes | Yes |
| Controlled checkbox | Yes | Yes | Yes |
| Icon button with tooltip, shortcut hint and busy state | Yes | Yes | Yes |
| Controlled text input and multiline editor | Yes | Yes | Yes |
| Action list, selection list, text table and tabs | Yes | Yes | Yes |
| Sidebar navigation | Yes | Yes | Yes |
| Modal edge sheet with a pinned header and scrolling body | Yes | Yes | Yes |
| Percentage progress | Yes | Yes | Yes |
| Rows, columns, borders and modal overlays | Yes | Yes | Yes |
| CUI content/fill, weighted growth, automatic shrinkage, alignment and spacing | Yes | Yes | Yes |
| Host-owned vertical scrolling and controlled start/end | Yes | Yes | Yes |
| Fixed basis, bounds, spacing and scroll offsets in cells | | Yes | |
| Fixed basis, bounds, spacing and scroll offsets in logical units | Yes | Yes | Yes |
| Semantic style roles and typed cached subviews | Yes | Yes | Yes |
| `CellStyled` colors and `CellCanvas` drawing/input | | Yes | |
| Terminal screens/layers, pinned panels, status hints, exact viewport and fixed canvas | | Yes | |
| Packed RGBA image and logical-coordinate vector canvas | | | Yes |

## Layout and style

Layout follows the supported subset of CUI's box rules. Content uses the host's
measured extent. Fill and positive Grow weights share the remaining space from
zero, freezing children that reach their minimum or maximum. Fixed sizes never
grow or shrink. Content children shrink proportionally toward their minimum
content size; bounds can leave overflow, which the host clips.

Default alignment stretches columns and centers rows on their cross axis.
Intrinsic controls such as buttons keep their natural size. Explicit Start,
Center, End and Stretch select another placement. Rows, columns, sizing,
spacing, borders and scrolling form the shared layout subset; CUI's grid and
wrapped-row APIs are not exposed by this contract.

Use `Measure.Logical` for portable fixed sizes, bounds, spacing and scroll
requests. GUI uses CUI logical points. TUI maps 8 logical units to a column and
16 to a row, rounding positive distances upward; weighted allocations round
shared boundaries so no cells are lost. `Measure.Cells` is an explicit terminal
extension. Semantic Tight/Normal/Loose spacing corresponds to 4/8/16 logical
units and adapts to the host's resolution.

## Shared themes

Both hosts accept `Fpas.Ui.Themes.Theme`. `Themes.Light(Themes.Accent.Blue)`
and `Themes.Dark(Themes.Accent.Blue)` select CUI palettes; omit the accent for
neutral. The accent families are Neutral, Violet, Blue, Green, Amber, Orange,
Coral and Rose. A default Theme is neutral light.

The shared Palette contains CUI's semantic straight-alpha colors. Customize it
with a record update on Theme.Colors, preserving an opaque Background. Keep
themes and colors unchanged while retained by a host. GUI applies this palette
to native CUI, including its accent behavior. TUI derives widget, focus,
selection, disabled and error colors from the same tokens and composites alpha
against the containing surface. Typography, round corners, shadows and animation
are native GUI presentation; terminal cells approximate the relevant states.
TUI requires no native CUI or graphical dependencies.

Pass the same `ThemeFor(Model): Themes.Theme` callback to either host. Fixed
themes go in GUI Options.Theme or the TUI Theme argument. Prefer Styled and
button variants over explicit terminal colors. Explicit cell canvases can
convert a palette color with Themes.Rgb(Color, OpaqueBackground).

`Layout.Viewport` expresses width and height independently as compact, regular
or expanded. The host also supplies `LogicalWidth` and `LogicalHeight` when it
knows the measured size. The terminal maps a column to 8 logical units and a row
to 16. `TerminalExtensions.Views.Viewport`
resolves a terminal subtree from exact `Terminal.Viewport` columns and rows.
The terminal host supplies this context to Prepare before any drawing.

`Styled` and button variants select semantic theme roles. Supply a theme key
or the relevant theme values as typed `CachedSubview` dependencies when
building a subtree depends on them. A host passes its last successfully
presented `Prepared` view to `Prepare`. The validator resolves each deferred
node once, reuses the previous subtree when its typed dependencies compare
equal, and validates it again against the current tree. Use the returned
`Subviews` for presentation. Retain it as the next cache after successful
presentation.

## Focus and activation

Focus follows stable control identity. Tab and Shift+Tab traverse enabled
focus stops in view order. Enter activates a focused action or the selected
item of a collection that supports activation. Space also activates buttons
and toggles checkboxes when a text editor does not own the keystroke. A
disabled control receives no user action or keyboard focus.

Selection lists and tables report key changes separately from activation.
Arrow navigation skips disabled items. `CanActivate` permits an item to stay
selectable while suppressing its activation. An enabled selection change
proposes a new key once. The selected key may be null. Tabs use the requested
key when enabled and otherwise use the first enabled tab. With no enabled
tabs they show no panel. The fallback only affects presentation.

The top modal owns focus and pointer input. It tries its initial focus ID,
then the first enabled focus stop. Tab remains within the modal. Escape
invokes its dismissal callback when present. Closing a modal restores
the prior focus if it is still available.

## Sidebar navigation

`Fpas.Ui.Navigation.Sidebar` takes ordered sections of keyed destinations and
the active key from the model. An empty section title omits the heading. Keys
are unique across sections. Each enabled destination is a focus stop. Tab and
Shift+Tab move focus, while a click, Enter or Space sends the item's message.
Focus can move independently of the active page. A null active key clears the
selection, and an unknown key is rejected during preflight.

Use `Navigation.ItemId(SidebarId, Key)` to address a destination for focus. The
GUI uses CUI's native navigation rows and shared animated highlight. The TUI
uses themed text rows. Both hosts display badge text, and the GUI can show a
glyph from the CUI icon font. Use the normal layout and scroll view to place or
size the sidebar.

## Sheet and drawer

`Fpas.Ui.Sheets.Sheet` takes a base view, a sheet body, model-owned visibility,
and a dismissal message. Keep the sheet in the view while closed to let the
GUI animate the body out. Give the sheet and its descendants distinct IDs.

`Sheets.Options` sets the title, optional description, edge and size. The
default is a right sheet, 400 logical units wide. Size is width for left and
right edges and height for top and bottom edges. Both hosts constrain the
panel to the viewport and scroll the body beneath its pinned header.

The open sheet owns keyboard focus and pointer input. It tries the requested
initial focus and falls back to its close button. Use `Sheets.CloseId(Id)` to
address that button. The button, a click on the dimmed backdrop, or Escape
proposes dismissal. A focused editor gets Escape first to collapse its
selection or leave editing. When the model closes the sheet, the host
restores prior focus if available. GUI uses CUI's slide transition. TUI
draws the sheet directly in cells and supports Page Up and Page Down for
the body when the focused control leaves those keys available.

## Button

`Fpas.Ui.Buttons.Button` takes a stable ID, a literal label, an activation
message and optional `Buttons.Options`. The default uses CUI's secondary
appearance. Options select the variant, control size, leading and trailing
icons, disabled and busy states, selected appearance, foreground color,
tooltip and shortcut hint. `Width` uses `Layout.Extent.Content`, `Fill` or
`Fixed` with an explicit measure. It always describes horizontal width,
including inside columns.

Tab focuses an enabled button. Click, Enter or Space proposes its message.
Loading keeps the label and focus, shows a spinner in GUI or an ellipsis in
TUI, and suppresses activation. Label and icon changes keep the same identity.
Applications bind the hinted shortcut separately.

GUI uses the CUI icon font. Set `TerminalIcon` and `TerminalIconRight` for
cell-based equivalents, or leave them empty to show just the label. Small and
medium buttons occupy one terminal row. Large buttons use two rows and wider
padding. TUI shows the tooltip and shortcut on the last row while hovered.
The existing `Elements.Button` constructor uses the same host implementations.

## Button group

`Fpas.Ui.ButtonGroups.ButtonGroup` takes direct button elements in display
order, including icon, copy and hold buttons. It puts them in one horizontal frame with a
divider between each pair. Default buttons use the ghost appearance inside
the group. Explicit variants and each button's own state and message still
apply. The group has no focus stop of its own. Tab visits enabled members
in order. Each member keeps its own activation gesture.

GUI draws the native CUI group. TUI uses a cell outline and reserves one
column for each divider.

## Icon button

`Fpas.Ui.IconButtons.IconButton` takes a stable ID, a CUI font glyph, a
tooltip that names the action, and the activation message. The ghost style
is the default. Options set the style, size, selected state, input permission,
busy state and shortcut hint. Bind the shortcut separately if it should run
the action.

Tab focuses an enabled icon button. Click, Enter or Space proposes its message.
A busy button keeps focus and suppresses activation. Changing the glyph or
tooltip preserves the button's identity. GUI draws the glyph with CUI. TUI
uses the supplied terminal glyph or the tooltip text, shows an ellipsis while
busy, and puts the hover hint on the last terminal row.

## Copy button

`Fpas.Ui.CopyButtons.CopyButton` takes a stable ID and literal clipboard
text. Tab focuses the button. Click, Enter or Space submits the text and
shows a check mark for 1.6 seconds without moving focus. An optional
`OnCopied` factory supplies a message after the host submits the request.

GUI uses CUI's platform clipboard provider. TUI sends the text as an OSC 52
write to the clipboard of the connected terminal. The terminal must support
and allow this operation. TUI feedback confirms that the request was sent.
Headless TUI sessions display the button disabled when no provider is attached.

## Hold to confirm

`Fpas.Ui.HoldButtons.HoldButton` takes a stable ID, a literal caption and a
message. Its options select a positive duration in milliseconds, the variant,
enabled state and optional GUI and terminal icons. The default is 1,200 ms
and the destructive variant.

GUI uses CUI's fill animation and proposes the message once when a mouse,
Space or Enter hold completes. TUI fills the button in cells and confirms on
left-button release over the same control after the required duration. Moving
off the control or releasing early cancels the hold. Terminal key input does
not carry key release, so a tap, repeated key presses or a synthetic click
cannot confirm this action.

## Split button

`Fpas.Ui.SplitButtons.SplitButton` takes a stable main-action ID, a literal
caption, its message and a nonempty array of keyed menu items. Each item has
a caption, message and optional GUI and terminal icons. Options set the
button variant, size, main icon and enabled state. Primary is the default.

Tab visits the main action and the chevron separately. Click, Enter or Space
activates the focused half. Down on the chevron opens the menu. Up and Down
move through its actions, Enter chooses, and Escape or an outside click
dismisses it. Closing returns keyboard focus to the trigger. The host owns
menu visibility, and choosing an item proposes its message once.

Use `SplitButtons.MenuId(Id)` to request chevron focus and
`SplitButtons.ItemId(Id, Key)` to inspect an item. GUI draws CUI's native
popup. TUI places a cell menu against the trailing edge of the split button
and confines input to it while open. Long terminal menus support wheel and
Page Up/Page Down scrolling.

## Text editing and scrolling

Both editor kinds take model-owned text and a directional `EditorPosition`.
The caret and anchor are UTF-16 offsets at .NET grapheme boundaries, including
the end of the string. Edits propose a replacement document and position.
Selection-only changes use the navigation callback. Use LF for multiline
breaks. When accepting input, normalize line breaks to LF for a multiline
editor or spaces for a single-line editor before computing the proposed
selection. The CUI adapter converts UTF-16 positions to UTF-8 byte offsets and
maps native positions back to grapheme boundaries. Visual scroll offsets,
preferred columns and text measurement belong to the host.

A controlled scroll view clamps the requested position to the current range.
`End` follows the end as content grows. User input proposes a new position
with the resolved offset and maximum in explicit host units. Layout and
content changes recompute the visible offset without emitting messages.

## Host preflight

Call `Validation.Prepare` with the host's profile before presenting any node.
It validates the root, modal layers, the active tab panel and resolved cached
subtrees. It reports an `ArgumentException` with the control ID or tree path
for incompatible units or extensions, invalid document positions, unknown
selections, duplicate keys or IDs, and malformed options. Apply any additional
adapter-specific limits before drawing. Draw and input callbacks run at their
respective stages after preflight.

`CellCanvas` uses coordinates relative to the visible cell area. Image bytes
and vector operations describe content, with image display size and vector
coordinates in logical units. The host owns native textures and rendering
resources.
