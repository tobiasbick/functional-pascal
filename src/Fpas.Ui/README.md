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
| Button variants and controlled checkbox | Yes | Yes | Yes |
| Controlled text input and multiline editor | Yes | Yes | Yes |
| Action list, selection list, text table and tabs | Yes | Yes | Yes |
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
or expanded. The host maps its measurements to these classes. Applications
that depend on exact rows, cells or GUI dimensions can choose separate views
with the same model and update function. `TerminalExtensions.Views.Viewport`
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
