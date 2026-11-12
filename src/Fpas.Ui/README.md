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
case-sensitive. Sidebar, dropdown and split-menu keys are unique within their own group.
An item key can repeat in another group. Keep view records,
arrays and callbacks unchanged for as long as the host retains
them. Replace changed data on a subsequent view.

## Capability profiles

`Fpas.Ui.Validation.Prepare` checks the following profiles. The table defines
the contract accepted by the validator. The host references describe rendering
and input support. [GUI adapter](../Fpas.Ui.Gui/README.md),
[TUI adapter](../Fpas.Ui.Tui/README.md).

| Widget or option | Portable | Terminal | Graphical |
| --- | --- | --- | --- |
| Text and titled or plain divider | Yes | Yes | Yes |
| Button variants, sizes, icons, busy state and content/fill/fixed width | Yes | Yes | Yes |
| Button group with a shared outline | Yes | Yes | Yes |
| Dropdown menus, command states, hints, sections and separators | Yes | Yes | Yes |
| Context menu attached to a component | | | Yes |
| Reactions and user menus | | | Yes |
| Popover and confirmation popover anchored to a button | | | Yes |
| Tooltip attached to a single control | | | Yes |
| Single-line text fields with labels, validation, affixes and password reveal | | | Yes |
| Search field and wrapping multiline text area with affixes | | | Yes |
| Single-selection segmented control, sizes, icons and full width | Yes | Yes | Yes |
| Copy button with transient feedback | Yes | Yes | Yes |
| Icon button with tooltip, shortcut hint and busy state | Yes | Yes | Yes |
| Sidebar navigation | Yes | Yes | Yes |
| Native table of contents and scroll anchors | | | Yes |
| Native responsive grid, justification, cards, badges, metrics, avatar groups and empty states | | | Yes |
| Tabs, breadcrumb, pagination and stepper | | | Yes |
| Modal edge sheet with a pinned header and scrolling body | Yes | Yes | Yes |
| Centered modal dialog and action footer | | | Yes |
| Banners, countdowns and onboarding checklists | | | Yes |
| Alerts, progress bars, spinners, meters, rings, skeletons and timeline entries | | | Yes |
| Hold, confirmation, status, theme switcher and theme toggle actions | Yes | Yes | Yes |
| Rows, columns and borders | Yes | Yes | Yes |
| Inline links, avatars and hover cards | | | Yes |
| Dropzones, avatar uploads and signature pads | | | Yes |
| Audio knobs, faders and level meters | | | Yes |
| Selection chips, whole-star ratings and wrapping rows | | | Yes |
| CUI content/fill, weighted growth, automatic shrinkage, alignment and spacing | Yes | Yes | Yes |
| Host-owned vertical scrolling and controlled start/end | Yes | Yes | Yes |
| Fixed basis, bounds, spacing and scroll offsets in cells | | Yes | |
| Fixed basis, bounds, spacing and scroll offsets in logical units | Yes | Yes | Yes |
| Percentage basis and button or skeleton width | | | Yes |
| Semantic style roles and typed cached subviews | Yes | Yes | Yes |
| `CellStyled` colors, screens, shortcuts and exact viewport | | Yes | |

## Layout and style

`Flow.Clip` controls native clipping at graphical stack bounds. By default,
ordinary stacks clip and wrapping rows allow paint to extend past the bounds.
Set it to false to keep strokes and shadows visible inside a containing viewport.

`Role.DisplayValue` uses the graphical display face at 22 logical points
with tabular digits.

Layout follows the supported subset of CUI's box rules. Content uses the host's
measured extent. Fill and positive Grow weights share the remaining space from
zero, freezing children that reach their minimum or maximum. Fixed sizes never
grow or shrink. GUI also supports `Extent.Percent(Value)` from zero through 100,
resolved by CUI against the parent's inner main-axis extent. Percentages cannot
grow and respect explicit bounds. Terminal and portable profiles reject them.
Content children shrink proportionally toward their minimum
content size; bounds can leave overflow, which the host clips.

Default alignment stretches columns and centers rows on their cross axis.
Intrinsic controls such as buttons keep their natural size. Explicit Start,
Center, End and Stretch select another placement. Rows, columns, sizing,
spacing, borders and scrolling form the shared layout subset. A graphical row
can opt into native line wrapping with `Flow.Wrap`. The graphical
`Role.Label` uses the weight and line metrics of `cui_label`.

Use `Measure.Logical` for portable fixed sizes, bounds, spacing and scroll
requests. GUI uses CUI logical points. TUI maps 8 logical units to a column and
16 to a row, rounding positive distances upward; weighted allocations round
shared boundaries so no cells are lost. `Measure.Cells` is an explicit terminal
extension. Semantic Tight/Normal/Loose spacing corresponds to 4/8/16 logical
units and adapts to the host's resolution.
When both minimum and maximum bounds are supplied, use the same measure kind
for both: `Logical` or `Cells`.

## Native grids and surfaces

`Fpas.Ui.Grids.Grid` uses CUI's equal-width grid. Set the maximum column count,
the minimum logical column width and the gap. CUI reflows the grid using the
space offered by its parent and sizes each row to its tallest cell. On
graphical rows and columns, `Layout.Flow.Justify` distributes free space
using Start, Center, End, Between or Around.

- `Fpas.Ui.Cards.Card` draws a native surface around child elements. Options
  set dimensions, padding, gap, axis, wrapping, muted fill and selection.
  An optional activation message makes the card hoverable and focusable.
- `Fpas.Ui.Badges.Badge` draws a literal label with an optional glyph, dot,
  solid fill and semantic tone.
- `Fpas.Ui.Metrics.Metric` shows a formatted value with an optional delta and
  sparkline. CUI selects the trend color from the sign and `LowerIsBetter`.
- `Fpas.Ui.AvatarGroups.AvatarGroup` takes ordered names and a visible limit.
  CUI draws the overlapping initials and remaining count.
- `Fpas.Ui.EmptyStates.EmptyState` centers a heading, optional description
  and icon. Supply the action label and message factory together. Use
  `EmptyStates.ActionId(Id)` to address the action for focus.

These controls and non-default justification require the graphical profile.
Overview demonstrates the metrics and team grid. Feedback demonstrates the
two-column cards, status badges, avatar group and empty-state action.

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
button variants over explicit terminal colors. Terminal styling can convert
a palette color with Themes.Rgb(Color, OpaqueBackground).

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
focus stops in view order. Enter and typed Space activate the focused action.
A disabled control receives no user action or keyboard focus. Theme selectors
and split menus provide their own arrow-key navigation.

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
`Fixed` with an explicit measure; GUI additionally accepts `Percent(Value)`.
It always describes horizontal width, including inside columns.

Tab focuses an enabled button. Click, Enter or Space proposes its message.
Loading keeps the label and focus, shows a spinner in GUI or an ellipsis in
TUI, and suppresses activation. Label and icon changes keep the same identity.
Applications bind the hinted shortcut separately.

GUI uses the CUI icon font. Set `TerminalIcon` and `TerminalIconRight` for
cell-based equivalents, or leave them empty to show just the label. Small and
medium buttons occupy one terminal row. Large buttons use two rows and wider
padding. TUI shows the tooltip and shortcut on the last row while hovered.

## Button group

`Fpas.Ui.ButtonGroups.ButtonGroup` takes direct button elements in display
order, including icon, copy and hold buttons. It puts them in one horizontal frame with a
divider between each pair. Default buttons use the ghost appearance inside
the group. Explicit variants and each button's own state and message still
apply. The group has no focus stop of its own. Tab visits enabled members
in order. Each member keeps its own activation gesture.

GUI draws the native CUI group. TUI uses a cell outline and reserves one
column for each divider.

## Segmented control

`Fpas.Ui.SegmentedControls.SegmentedControl` takes a stable group ID, one through
32 `Item` choices, a model-owned selected index and an `OnChange` mapper. Array
positions define identity, as in CUI; update the selected index together with
any changes to the choices. Labels are literal and provide accessible names.
Items can supply CUI icons and explicit terminal equivalents; `IconOnly` hides
the GUI caption while the terminal keeps the readable label.

`Options` selects the three native control sizes, full-width allocation and
input permission. Full-width choices share the horizontal allocation equally.
Tab visits each choice without changing selection. Left/Right move focus and
propose the adjacent choice without wrapping. Pointer, Enter and Space propose
a focused choice; activating the selected choice proposes nothing. The model
may reject a proposal. Use `ItemId(Id, Index)` for individual focus requests.

GUI delegates layout, drawing and animation to CUI's segmented primitive. TUI
uses joined cell buttons with the same model and input contract; large segments
occupy two rows and the other sizes one. The theme switcher reuses these terminal
segments with named appearance choices. Both galleries clone the upstream view
and compact range examples on the Buttons page.

## Toggle group

`Fpas.Ui.ToggleGroups.ToggleGroup` takes a stable group ID, one through 32
positional items, one model-owned selection flag per item, and an `OnChange`
mapper. Independent groups toggle each item separately. Exclusive groups
require one selected item and propose a replacement selection when another
item is activated. Each proposal contains a fresh array of flags.

`Options` sets the control size, selection mode, icon-only presentation and
input permission. Items with glyphs use their labels as tooltips in icon-only
mode. Use `ItemId(Id, Index)` to focus an item, or the group ID to focus the
first item. Pointer, Enter and Space activate an item. Tab moves focus, and
each horizontal arrow press moves focus one item within the group.

Toggle groups require the graphical profile. CUI draws and handles both
groups on the GUI gallery's Actions page.

## Reactions

`Fpas.Ui.Reactions.Reactions` takes an ID, an ordered list of model-owned
reactions, and an `OnChange` mapper. Each item has an accessible name, a CUI
glyph, a count and an activation flag. The mapper receives the changed position
and CUI's proposed item. Retain the proposal in the model to apply it. CUI
animates the displayed count and selection. Use `ItemId(Id, Index)` to focus an
item. Reactions require the graphical profile.

## Speed dial

`Fpas.Ui.SpeedDials.SpeedDial` takes one through sixteen caption and glyph
pairs. CUI lays out and animates the floating actions. Choosing an action
passes its zero-based index to the supplied mapper. Use `ItemId` to inspect
or focus a visible action. Escape closes the dial and restores focus from a
focused action to the trigger. Speed dials require the graphical profile.

## Swipe actions

`Fpas.Ui.SwipeActions.SwipeItem` wraps a foreground view in CUI's horizontal
swipe row. Each side has ordered actions with a caption, optional glyph and
semantic tone. The callback receives the side and zero-based action index.
Drag with the mouse or scroll horizontally over the row to reveal the actions.
The row can also show a leading glyph and trailing annotation. Swipe rows
require the graphical profile.

## User menu

`Fpas.Ui.UserMenus.UserMenu` takes a stable ID, an account name and the same
`Fpas.Ui.Menus.Entry<TMsg>` commands and decorations as a dropdown. CUI draws
the initials avatar, optional status and detail, trigger and popup. `Options`
sets compact presentation, popup placement and width, and input permission.
Use the pointer, Enter or Space to open the menu. Choosing a command dispatches
its message. Escape dismisses the popup and returns focus to the trigger.
User menus require the graphical profile.

## Checkbox, switch and radio controls

`Fpas.Ui.Checkboxes.Checkbox` and `Fpas.Ui.Switches.Switch` take a stable
ID, a literal label, a model-owned Boolean and an `OnChange` mapper. The
whole row accepts pointer input. Tab focuses enabled rows, and Space or
Enter proposes the opposite value. Both controls accept secondary
`Description` text. A checked checkbox can use `Indeterminate` to show a
dash.

`Fpas.Ui.RadioButtons.RadioButton` proposes selecting an unselected row
through its `OnSelect` mapper. `Fpas.Ui.RadioGroups.RadioGroup` arranges
native radio rows in a column. Its model owns a selected index, or `-1`
before a choice is made. Each item can have a description and its own
input permission. Activating the selected item proposes no change. Use
`RadioGroups.ItemId(Id, Index)` to focus or inspect a specific row.

These controls require the graphical profile. The GUI gallery's Selection
page shows the original notification checkboxes, connectivity switches
and save-frequency radio group.

## Radio cards

`Fpas.Ui.RadioCards.RadioCards` takes a stable ID, choices with a title
and optional description, icon and badge, a model-owned index, and an
`OnChange` mapper. Use `-1` for an empty selection. CUI arranges the cards
in a grid that reduces the column count as space runs out. `Options.Columns`
sets the maximum when it is positive. Click, Enter or Space selects a card.
Arrow keys move focus and selection together. Focus a particular card with
`RadioCards.ItemId(Id, Index)` or focus the group to reach the selected card.

## Slider and range slider

`Fpas.Ui.Sliders.Slider` proposes a single model-owned value using pointer
input or the keyboard. A positive `Step` snaps changes to an increment.
A zero step allows continuous dragging. Arrows adjust the value, Shift
accelerates the adjustment, and Home and End move toward the bounds.
`ShowValue`, `DecimalPlaces` and `Percent` control the value caption.

`Fpas.Ui.RangeSliders.RangeSlider` takes a model-owned `Interval` with two
ordered endpoints. The native range track moves the nearest endpoint on
press and keeps the two endpoints ordered during a drag. Tab focuses the
track, Space switches the active endpoint, and the slider keys adjust it.
Both sliders use finite, increasing bounds and values within those bounds.
Radio cards and both sliders require the graphical profile.

## Ruler and histogram sliders

`Fpas.Ui.RulerSliders.RulerSlider` displays a movable tape under a fixed
marker. The model supplies a bounded value. Options configure the tick
step, spacing, major tick interval, width, decimal places and unit. Native
input includes dragging, the wheel, and arrow keys. Shift+arrow moves one
major tick interval.

`Fpas.Ui.HistogramSliders.HistogramSlider` overlays two range handles on
nonempty, finite nonnegative bin heights. It uses the same model-owned
`Fpas.Ui.RangeSliders.Interval` as a range slider. Space switches the active
handle for keyboard adjustment. Options configure bar height and the
endpoint captions. Both sliders require the graphical profile.

## Link, avatar and hover card

`Fpas.Ui.Links.Link` draws CUI's native inline link and proposes the supplied message on
click, Enter or Space. The application handles navigation in its update.

`Fpas.Ui.Avatars.Avatar` draws initials or loads the optional model-owned image path. Size,
round or rounded-square shape and the presence dot use native CUI options.
Use online, busy or away for colored presence. An empty status hides the dot.

`Fpas.Ui.HoverCards.HoverCard` attaches a nonmodal body to one interactive control. CUI
opens it after the hover delay and keeps it open while the pointer is over
the trigger or card. Escape closes it until the pointer leaves the trigger.
Width and preferred placement are optional. All three widgets require
the graphical profile.

## Capture controls

`Fpas.Ui.Dropzones.Dropzone` receives a detached array of accepted paths from CUI. Its options
set the title, description, extension filter, icon and compact or tall
appearance. `Fpas.Ui.AvatarUploads.AvatarUpload` displays initials or a decoded image from the model's
path. Store the path proposed by a successful drop to accept the new image.

Both controls can map browse activation to an application message.
`Fpas.Ui.Gui.FileDialogs.OpenFile` is the graphical host command for opening the native
file chooser and mapping a matching selection to a message. Cancellation
delivers no message.

`Fpas.Ui.SignaturePads.SignaturePad` lets CUI own the ink and Clear action. An optional callback receives
a detached snapshot on drawing and clearing. Each stroke end is an exclusive
index into the points array. The points use CUI's logical drawing space.
`SignaturePads.ClearId(Id)` addresses the native Clear button while it is visible.
All three controls require the graphical profile.

## Audio controls

`Fpas.Ui.Knobs.Knob` and `Fpas.Ui.Faders.Fader` take a stable ID, a literal caption,
a finite value within increasing bounds, and an OnChange mapper. CUI
handles dragging, the wheel on the focused control and keyboard adjustment.
Double-click, Delete or Backspace restores the configured reset value.
Shift makes pointer dragging finer and increases the arrow-key step.

Knob options include diameter, logarithmic scaling for positive ranges,
a bipolar arc and an arc color. Faders can run in either direction and
show up to 32 tick labels and up to eight channels of metering beside the track.
Both controls support continuous or stepped values, decimal precision,
an optional unit and a hidden value caption.

`Fpas.Ui.LevelMeters.LevelMeter` takes one to eight nonnegative, finite
linear sample peaks. A peak of 1.0 represents 0 dBFS. CUI draws the bars,
decay, peak hold and a latched clip light. Options control the dBFS bounds,
dimensions, orientation, scale, segments and ballistics. Supply
OnClipCleared to receive a message when the clip light is clicked.
All three controls require the graphical profile.

## Chip and rating

`Fpas.Ui.Chips.Chip` takes a stable ID, literal caption, model-owned Boolean
and an `OnChange` mapper. Click, Enter or Space proposes toggling the chip.
The optional icon appears when unselected, and CUI draws its check mark when
selected. Use a wrapping row for groups of independent chips.

`Fpas.Ui.Ratings.Rating` takes a stable ID, a name for inspection, a
model-owned whole-star value and an `OnChange` mapper. It defaults to five
stars and accepts a maximum from one through sixteen. Clicking a star
proposes its number, and clicking the selected star proposes zero. CUI
operates the rating by pointer. Both widgets require the graphical profile.

## Tabs, breadcrumb and pagination

`Fpas.Ui.Tabs.Tabs` takes a stable ID, up to 32 positional tabs, a model-owned
zero-based selection and an `OnChange` mapper. Native tabs support underline
or pill indicators, optional icons and full-width allocation. Click or use
Left and Right to change selection. CUI scrolls an overflowing strip
horizontally. The application chooses the selected tab's content in its view.
Use `Tabs.ItemId(Id, Index)` to focus a particular tab.

`Fpas.Ui.Breadcrumbs.Breadcrumb` takes a stable trail ID, literal captions
and an `OnNavigate` mapper. Activating an ancestor proposes its zero-based
position. The last caption names the current location. Positional IDs
allow repeated captions. Use `Breadcrumbs.ItemId(Id, Index)` to focus an
ancestor.

`Fpas.Ui.Paginations.Pagination` takes a stable pager ID, a zero-based
selected page, a positive page count and an `OnChange` mapper. CUI displays
one-based numbers and chooses which page buttons and ellipses fit.
Previous and Next are disabled at the corresponding bounds. Tab focuses
enabled breadcrumb links and page buttons, and Enter or Space activates
them. These three controls require the graphical profile.

## Stepper

`Fpas.Ui.Steppers.Stepper` takes a stable ID, a nonempty sequence of literal
captions and a model-owned zero-based step index. CUI marks the preceding steps
complete. Pass the number of steps to mark the whole sequence complete. When
space is tight, CUI keeps the current caption and shows the other step markers.
The stepper is a read-only graphical display. Use separate controls to change
the model's progress.

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

## Confirm button

`Fpas.Ui.ConfirmButtons.ConfirmButton` takes a stable ID, a literal resting caption
and the message proposed after confirmation. Options select the confirmation
caption, positive timeout in milliseconds, resting variant, size, enabled state,
and GUI/terminal icons. Defaults are `Click to confirm`, 3,000 ms, secondary and
medium. The armed appearance is destructive.

The first click, Enter or Space arms the button; a second activation within the
window sends the message once. Escape while focused or hovered, a click outside,
or timeout disarms it. Disabling, removing or covering it with a modal also
cancels confirmation. Changing keyboard focus or captions alone keeps the
pending confirmation. GUI uses native CUI animation and countdown; TUI appends
the remaining seconds to the caption. Both retain the same focus identity.

## Status button

`Fpas.Ui.StatusButtons.StatusButton` takes a stable ID, a literal idle caption,
the model's `Status` (`Idle`, `Busy`, `Success`, `Error`) and its action message.
Options select feedback captions, idle icons, variant, size and enabled state.
Defaults are primary/medium, the idle caption followed by `…` while busy,
`Done` on success and `Failed` on error. An explicit empty busy caption stays empty.

Click, Enter or Space proposes the message only while idle. Other work states
remain focusable and reject activation. Disabling rejects both input and focus.
The application owns every state change, including starting work and returning
to idle; the widget has no automatic completion or reset timer. Recreating the
view preserves the model-owned status. Captions and state changes keep the same
identity. Snapshots use `status-button` and `idle`/`busy`/`success`/`error` values.

GUI uses native CUI animated faces, a busy spinner and result icons. TUI uses a
static busy mark, check/error symbols and semantic success/danger colors, with
distinct focus and hover styles. The galleries demonstrate model transitions
using Complete, Fail and Reset buttons.

## Theme switcher

`Fpas.Ui.ThemeSwitchers.ThemeSwitcher` takes a stable group ID, a model-owned
`Selection` (`Light`, `Dark`, `System`) and an `OnChange` mapper. Options select
`WithSystem` (default false) and `Enabled` (default true). Selecting System while
it is unavailable is a preflight error. The native selector uses CUI's small
sun/moon/monitor segments; TUI uses a joined row of named text segments.

Each segment is a focus stop. Tab/Shift+Tab move focus without selecting;
Left/Right move and propose the neighboring choice without wrapping. Click,
Enter or typed Space proposes the focused choice if it differs from the model.
Preparing a view and activating the selected choice do not call the mapper.
Disabled or modal-covered segments reject input and focus. Use
`ThemeSwitchers.ItemId(Id, Choice)` for a segment focus request on either host.

Apply the accepted choice through the existing `ThemeFor` callback. The widget
does not replace the palette itself, including when the model rejects a proposal.
`ThemeSwitchers.Resolve(Selected, SystemAppearance)` returns the concrete
`Themes.Mode` for selecting a light/dark theme while preserving the application's
accent and palette customization. Supply `SystemAppearance` from the application's
appearance provider. As in CUI, the System segment itself does not detect OS or
terminal settings. Provider changes update the resolved theme while System stays
selected. The galleries include an explicitly simulated provider control.

GUI snapshots include a non-interactive `theme-switcher` group with a
`light`/`dark`/`system` value and keyed `theme-choice` segments. TUI segments use
the IDs from `ItemId`. Segment values are `true` for the model's choice and
`false` for the others; focus remains independent of selection.

## Theme toggle

`Fpas.Ui.ThemeToggles.ThemeToggle(Id, Appearance, OnChange, Enabled := true)`
offers a compact light/dark action. Pass the model's concrete `Themes.Mode`;
resolve a System selection with `ThemeSwitchers.Resolve` first. Activation
proposes the opposite mode through `OnChange`. Preparing a view never calls
the mapper. The application accepts or rejects the proposal in Update and
applies its palette through `ThemeFor`, retaining custom colors and accents.

GUI uses CUI's sun/moon icon button; TUI shows a sun/moon and a readable target
caption. Both have the same stable identity, accessible target name, Enter,
Space and pointer activation. Disabled and modal-covered actions reject input
and focus. Snapshots use `theme-toggle` with the current `light`/`dark` value.
The gallery's toggle replaces System with an explicit light/dark choice.

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

## Dropdown menu

`Fpas.Ui.Menus.Menu` takes a trigger ID, a literal caption, ordered `Entry`
content and optional `Options`. Use `Entry.Action(new Item<TMsg>(Key, Label,
Message, ...))` for commands, `Entry.Label(Text)` for section captions and
`Entry.Separator` for rules. Command keys define identity independently of
caption and position; `Menus.ItemId(Id, Key)` returns the inspection identity.

Commands can have a native icon, a terminal equivalent, a shortcut hint,
input permission, a danger color and a model-owned check mark. Shortcut hints
are display-only; bind application shortcuts through the ordinary input API.
Choosing a checked command still sends its message; Update owns any change
to its checked state. Noninteractive rows never send messages.

Options select the native trigger variant, icon-only presentation, preferred
placement and logical popup width. Zero width uses content sizing with CUI's
180-unit minimum. GUI uses CUI's menu primitives for layout, paint, input,
animation and visibility. The caption remains the accessible name and tooltip
for an icon-only trigger; the native ellipsis is its default icon.

Down, Enter or pointer activation opens the focused trigger. Native CUI owns
the GUI highlight; Up/Down move it and Enter chooses. Escape, an outside click
or a choice closes the menu. Disabling or removing the trigger retires its
open popup. Empty dynamic menus can still be dismissed. TUI shares the
split-button popup foundation, starts on the first enabled command, skips
disabled commands and decorations during focus navigation, and uses cell
geometry for placement, width, clipping and scrolling. It keeps a readable
trigger caption when no terminal icon is supplied. Both galleries show
the original Options and More actions examples; their commands are harmless
model updates.

## Context menu

`Fpas.Ui.Menus.ContextMenu(Id, Target, Entries, Enabled)` attaches CUI's context
menu to the target's native root component. It returns that same target with
its layout, actions and state intact. `Entries` uses the dropdown's `Item` and
`Entry` types, including icons, shortcut hints, checked/disabled commands,
danger colors, section captions and separators. `Menus.ItemId` also applies.

CUI opens the menu at the pointer on right press, handles its native placement,
Up/Down and Enter, and closes it after selection, Escape or an outside click.
Opening requires a pointer. Interactive descendants retain their own hit areas;
attach a menu to the component that should receive the right click. Static
compositions can provide a whole right-click area. The wrapper adds no visual
container, positioning engine or application-owned open state. Disabling or
removing the menu closes its popup without retiring the target's widget state.

Snapshots use `context-menu` for the target area and `menu-item` for open
commands. The target value reports whether the popup accepts input. Context
menus require the graphical profile; terminal and portable preflight reject
them. The GUI gallery's Overlays page shows Cut, Copy, Paste and Delete as
harmless preview messages.

## Accordion and expandable card

`Fpas.Ui.Accordions.Accordion<TMsg>(Id, Title, Open, OnChange, Content, Enabled)`
and `Fpas.Ui.ExpandableCards.ExpandableCard<TMsg>(Id, Title, Open, OnChange,
Content, Settings)` use CUI's native header, chevron, layout and height spring.
`Open` belongs to the model; `OnChange` proposes the next boolean. Update may
accept or decline it. Stable IDs are independent of literal titles, including
`##` and `###`. Pointer activation, Enter and Space toggle the focused header.
Escape leaves expansion unchanged.

Card options add CUI's description, icon, badge and `Styles.Tone` badge color. `Enabled` gates
the header and body. CUI still draws outgoing content while closing, with input
disabled; the host omits those descendants from agent snapshots and focus
selection. The body's own control IDs remain independent of the header.
Snapshots use `accordion` and `expandable-card`, with the model's boolean in
`Value` and the native header's bounds and focus. CUI draws and sizes the whole
wrapper; no second disclosure layout or animation system is involved.

## Expandable text

`Fpas.Ui.ExpandableTexts.ExpandableText<TMsg>(Id, Text, Settings, OnToggle)`
uses CUI's wrapping text, fade and Show more / Show less link. CUI owns local
expansion; removing the element releases it. `OnToggle` optionally notifies the
application of native activation without duplicating that state in its model.
Options map to CUI's collapsed `Lines` (default three), font `Size` (zero uses
the body size), `Color`, `FadeTo` and input permission. Set `FadeTo` when the
surface differs from the theme background.

The noninteractive `expandable-text` snapshot reports native expansion in
`Value`. When the text exceeds the line limit, `ExpandableTexts.ToggleId(Id)`
identifies its `expandable-text-toggle` link for inspection and focus requests.
The link disappears when the text fits. Native CUI currently loses keyboard
focus after toggling ([CUI-006](../../docs/future/cui-upstream-bugs.md#cui-006));
Tab or an explicit focus request selects it again. All three disclosure widgets
require the graphical profile. Terminal and portable preflight reject them.

## Banner, countdown and checklist

`Fpas.Ui.Banners.Banner` displays a native announcement with model-owned visibility. Supply a
dismiss message to show the close button and update the model when it is used.
An action caption and message factory together enable the action. The native
banner animates when the application changes its visibility.

`Fpas.Ui.Countdowns.Countdown` draws CUI's rolling digits from the supplied remaining seconds.
Applications refresh that value on their own clock. Use compact tiles for a
short display, and hide days to fold them into hours.

`Fpas.Ui.Checklists.Checklist` renders ordered steps with model-owned completion. The first
incomplete step exposes its description and optional action. Clicking a circle
or activating it with the keyboard proposes a new completion flag. An action
button sends its step index, which the application can complete in its update.
All three widgets require the graphical profile.

## Alerts, progress and spinners

`Fpas.Ui.Alerts.Alert<TMsg>(Id, Title, Settings, OnDismiss)` draws CUI's inline
alert. Options provide `Description`, `Styles.Tone`, an optional icon glyph,
`Dismissible` and `Enabled`. Empty icons use CUI's tone-specific default. Titles
and descriptions are literal, including `##`. CUI owns dismissal: the alert
stays hidden while its stable ID remains in the view, even if its title changes.
Removing it for a presented frame releases that state. `OnDismiss` optionally
delivers one notification after the native action. Snapshots omit hidden alerts;
visible ones have role `alert`. `Alerts.DismissId(Id)` identifies the optional
`alert-dismiss` action for native pointer/Enter/Space input and explicit focus.

`Fpas.Ui.ProgressBars.ProgressBar<TMsg>(Id, Value, Settings)` displays a finite
fraction from zero through one. Options map to CUI's `Label`, `ShowValue`,
`Indeterminate`, `Styles.Tone` and logical track `Height` (zero uses the native
default). Indeterminate bars omit percentages. Use `Elements.Size` for width.
CUI owns interpolation and activity motion. The noninteractive `progress`
snapshot reports the invariant fraction or `indeterminate` in `Value`.

`Fpas.Ui.Spinners.Spinner<TMsg>(Id, Size := 0.0F)` uses CUI's activity animation,
theme color and logical diameter; zero selects its default. It has a
noninteractive `spinner` snapshot. Dimensions must be finite and nonnegative.
All three widgets require the graphical profile. The Feedback gallery shows
the original four alerts, upload and indexing bars, and 16/22/28-point spinners.

## Skeleton

`Fpas.Ui.Skeletons.Skeleton<TMsg>(Id, Width, Height := 0.0F)` draws CUI's loading
placeholder. Omitted width fills its parent. Use `Layout.Extent.Fixed` with a
logical measure or `Layout.Extent.Percent` for another width; `Content` is
rejected because a placeholder has no intrinsic content. Percentages are finite
values from zero through 100. Height must be finite and nonnegative; zero uses
CUI's 14-point default.

CUI owns rounding, shimmer, animation scheduling and reduced-motion behavior.
FPAS adds no animation loop or retained widget state. The GUI-only `skeleton`
snapshot is passive, with empty text and value. Feedback reproduces the original
Loading avatar, percentage text lines and 88-point body placeholder.

## Timeline item

`Fpas.Ui.TimelineItems.TimelineItem<TMsg>(Id, Title, Settings)` displays a
noninteractive native activity entry. The title must be nonempty. Options map
directly to CUI's optional literal `Time` and `Description`, icon glyph,
`Styles.Tone`, and `Last` flag. An empty icon uses CUI's dot. `Last` omits the
connector and trailing item space; set it on the final entry.

CUI owns text wrapping, title/time alignment, dot colors, icon and connector
painting. FPAS does not parse or format the time caption. All text is literal,
including `##`. The GUI-only `timeline-item` snapshot reports the title in
`Text` and the time caption in `Value`. Feedback reproduces the original three
Activity entries, including the final failed build without a connector.

## Progress ring

`Fpas.Ui.ProgressRings.ProgressRing<TMsg>(Id, Label, Value, Size := 0.0F)` draws
CUI's circular progress with a centered percentage and optional literal caption.
The fraction must be finite and between zero and one. The diameter must be
finite and nonnegative; zero selects CUI's 96-point default. The native ring
keeps its diameter rather than stretching across its container.

CUI owns numeric formatting, interpolation and painting. The stable ID is
independent of the caption and surrounding node positions; captions containing
`##` remain literal. Removing the ring releases its native interpolation state.
The noninteractive `progress-ring` snapshot reports the model fraction in
invariant notation. This widget requires the graphical profile. Feedback shows
the original Onboarding and Quota examples at 104 points.

## Meter

`Fpas.Ui.Meters.Meter<TMsg>(Id, Label, Used, Total, Unit := '')` displays native
capacity usage, for example `Meters.Meter<Msg>('storage', 'Storage', 18.4F, 20.0F, 'GB')`.
Usage and capacity must be finite and nonnegative. Usage may exceed capacity;
CUI clamps the visual fill. Zero capacity displays an empty track. CUI owns
numeric formatting, interpolation and its warning/danger thresholds. Labels and
units are literal, including `##`; use `Elements.Size` to constrain width.
The noninteractive `meter` snapshot reports invariant `Used/Total Unit` in
`Value`, omitting the space and suffix when Unit is empty. Removing the meter
releases its native interpolation state. Meter requires the graphical profile.

## Popover and confirmation popover

`Fpas.Ui.Popovers.Popover<TMsg>(Id, Trigger, Content, Open, OnDismiss, Settings)`
attaches a nonmodal native panel to a button. The trigger keeps its layout
and activation message. Use that message to toggle `Open` in the model. Escape
and a press outside propose `OnDismiss`. Actions in `Content` can close the
panel through their own updates. CUI fits the panel to the viewport around
the requested placement. Width, padding and gap use logical points.

`Fpas.Ui.ConfirmPopovers.ConfirmPopover<TMsg>` asks a question at the trigger
and draws CUI's affirmative and cancel buttons. The model opens the question
in response to the trigger. Confirm proposes `OnConfirm`. Cancel, Escape
and a press outside propose `OnDismiss`. Close the question in the update for
either outcome. The question opens with focus on its affirmative action and
requires a separate activation. Both popovers require the Graphical profile.

## Tooltip

`Fpas.Ui.Tooltips.Tooltip<TMsg>(Target, Text, Shortcut)` attaches native
pointer help to one button, checkbox, switch, radio button, chip or slider.
The target retains its layout, focus and activation. The optional shortcut
appears as a key hint. Bind the gesture separately in the application.

CUI shows a tooltip after about 550 ms of hover. Moving directly between
tooltipped controls shows the new hint immediately while the tooltip is
warm. The caption and shortcut must fit in 255 UTF-8 bytes together,
including six separator bytes when a shortcut is supplied. The caption
must be nonblank and cannot contain the separator `  ·  `. This wrapper
requires the Graphical profile.

## Dialog and action footer

`Fpas.Ui.Dialogs.Dialog<TMsg>(Id, Base, Content, Open, Settings, OnDismiss)`
presents a centered CUI modal above `Base`. The model controls `Open`.
Keep `Content` available during the exit transition. The host disables the
underlying view, moves focus into the dialog and restores focus on closure.
`InitialFocus` can target an enabled control in the body or footer.

CUI fits the requested width to the viewport. Escape, the native close
button and a click on the scrim propose `OnDismiss`. A focused text field
handles Escape before the dialog. `Persistent` suppresses Escape and scrim
dismissal. Set `CloseButton` to false to hide the native close action.

`Fpas.Ui.ActionFooters.ActionFooter<TMsg>(Children)` lays out the supplied
actions in CUI's right-aligned row with an eight-point top inset and an
eight-point gap. Cancel and Save can close a dialog through their ordinary
application messages. Both elements require the Graphical profile.

## Text field

`Fpas.Ui.TextFields.TextField<TMsg>(Id, Value, OnChange, Settings, OnSubmit)`
draws a CUI single-line editor. Set the caption, placeholder, leading icon,
hint, error, prefix and suffix through `Options`. An error replaces the hint.
The editable value excludes the prefix and suffix. Use shared layout options
to size the field.

CUI handles caret motion, selection, clipboard input and undo. The host
delivers an edit before the optional Enter submission, which receives the
same resulting text. An external replacement clears the old undo history
and moves the caret to the end of the new value. Read-only fields retain
focus and selection. Disabled fields block input to the editor and its
buttons.

Values are single lines bounded by `MaxUtf8Bytes`, excluding the terminator.
The default is 255 bytes. Password fields support up to 341 bytes so CUI can
measure the masked caret and selection. The native eye can reveal the value
on screen. GUI snapshots omit password values even when revealed. This
widget requires the Graphical profile.

## Search field

`Fpas.Ui.SearchFields.SearchField<TMsg>(Id, Value, OnChange, Settings, OnSubmit)`
uses CUI's search icon, placeholder and clear button. The query is a model-owned
single line. The native clear button empties it and returns focus to the field.
Enter can submit the current query. Options allow a custom icon, placeholder,
validation text, prefix and suffix. The query excludes the displayed affixes.

## Text area

`Fpas.Ui.TextAreas.TextArea<TMsg>(Id, Value, OnChange, Settings, OnSubmit)`
uses CUI's wrapping editor for model text with LF line endings. Enter inserts
LF. Ctrl+Enter submits the current text on Windows and Linux. Up and Down
move between visual lines. The wheel scrolls the text area and then its
enclosing viewport when the area reaches the end.

`Height` is in logical points and defaults to 96. Zero uses the native default
and a negative value fills the available height. The prefix and suffix
remain outside the editable text. Both editors share the model reconciliation,
submission ordering and UTF-8 capacity rules of the text field and require
the Graphical profile.

## Tag input

`Fpas.Ui.TagInputs.TagInput<TMsg>(Id, Tags, OnChange, Settings)` uses an ordered
array of model-owned tags. Enter or comma commits the draft. Backspace in
an empty draft removes the last tag, and each chip has a native remove button.
Tags are nonempty single lines without commas or surrounding ASCII spaces.
The native field supports up to 64 tags and 2047 UTF-8 bytes, including the
comma-space separators. Options can lower either limit. The remove buttons
use `TagInputs.RemoveId(Id, Index)` in GUI snapshots.

## One-time code input

`Fpas.Ui.OneTimeCodes.OneTimeCodeInput<TMsg>(Id, Value, OnChange, Settings, OnComplete)`
uses one to twelve native boxes for ASCII letters or digits. Typing appends,
Backspace removes the last character, and paste replaces the code. When a
user edit fills all boxes, the change message precedes the optional
completion message. A GUI snapshot exposes only the number of entered
characters.

## Password strength meter

`Fpas.Ui.PasswordStrengthMeters.PasswordStrengthMeter<TMsg>(Id, Password)`
displays CUI's heuristic score from zero to four and its corresponding
strength label. Its GUI snapshot contains the score and label. These three
widgets require the Graphical profile.

## Multi-select

`Fpas.Ui.MultiSelects.MultiSelect` takes `Selects.Item` choices and one
model-owned flag per choice. The native popup supports checkboxes, optional
search, and selecting or clearing the list. The field shows removable chips
and a count of extra selections. The callback receives a fresh array of flags
when CUI proposes a change. Multi-select requires the graphical profile.

## Color and emoji pickers

`Fpas.Ui.ColorPickers.ColorPicker` opens CUI's native swatch palette with
hue, saturation/value and eight presets. It reads and proposes a
`Fpas.Ui.Graphics.Color`. RGB edits preserve the model's alpha channel.

`Fpas.Ui.EmojiPickers.EmojiPicker` displays CUI's native search, group
tabs, skin tones, recent emoji and preview. It can be placed inline or
inside a `Popovers.Popover`. Its model-owned state holds the selected skin
tone and the space-separated recent list. A change carries the updated
state and, on selection, the complete emoji string. Both pickers require
the graphical profile.

## Calendar and date picker

`Fpas.Ui.Calendars.Calendar` shows CUI's single-month calendar inline.
`Fpas.Ui.DatePickers.DatePicker` opens it from a date field with an
optional label and placeholder. Both read a model-owned `System.DateOnly?`
and propose a `System.DateOnly` on selection. The application can clear the
selection by passing null. Each calendar retains its own displayed month.
Both widgets require the graphical profile.

## Date range

`Fpas.Ui.DateRangePickers.DateRangePicker` opens a native calendar with
optional presets and adjusts the number of months to the available width.
`Fpas.Ui.DateRangeCalendars.DateRangeCalendar` shows one or two months inline.
Both accept a model-owned
`Fpas.Ui.DateRanges.Value` containing `Empty`, `Started`, or `Complete` with
`System.DateOnly` endpoints. A complete range has inclusive, ordered endpoints.
The callback receives each proposed selection, so a picker and a calendar
can share one value, including a first endpoint that still needs an end date.
Each native calendar retains its own displayed month. Both widgets require
the graphical profile.

## Time picker, wheel picker and time dial

`Fpas.Ui.TimePickers.TimePicker` opens CUI's scrolling hour and minute
columns. It defaults to 24-hour display and five-minute choices. Enable
`TwelveHour` for an AM/PM column and set `MinuteStep` to change the
choices. `Fpas.Ui.TimeDials.TimeDial` uses an analog face with hour and
minute selectors. Both accept a model-owned `System.TimeOnly` at minute
precision and report the proposed time through the callback.

`Fpas.Ui.WheelPickers.WheelPicker` takes captions and a model-owned
zero-based index. CUI handles dragging, wheel input and arrow keys. Options
set the width, visible row count, wrapping and joined selection bands.
All three widgets require the graphical profile.

## Select

`Fpas.Ui.Selects.Select<TMsg>(Id, Items, Selected, OnChange, Settings)` uses
zero-based indices in model order. Use minus one for an unselected trigger.
Items supply a literal caption and an optional glyph. Enable `Searchable`
to filter the native popup. Choosing a row reports its original index and
returns focus to the trigger. Escape dismisses the popup. Visible option
rows use `Selects.OptionId(Id, Index)` in GUI snapshots.

## Number field

`Fpas.Ui.NumberFields.NumberField<TMsg>(Id, Value, Minimum, Maximum, OnChange, Settings)`
uses a finite model-owned double within inclusive bounds. Type a complete
finite number or adjust it with the native step buttons or arrow keys.
Shift with an arrow applies ten steps. Input is clamped to the bounds and
Enter or loss of focus displays the accepted value. `DecimalPlaces` sets
display precision. `Suffix` adds unit text beside the editor. The buttons
use `NumberFields.DecreaseId(Id)` and `NumberFields.IncreaseId(Id)` in GUI
snapshots. Both widgets require the Graphical profile.

## Scrolling

A controlled scroll view clamps the requested position to the current range.
`End` follows the end as content grows. User input proposes a new position
with the resolved offset and maximum in explicit host units. Layout and
content changes recompute the visible offset without emitting messages.

## Table of contents and scroll anchors

Add `Fpas.Ui.ScrollAnchors.ScrollAnchor<TMsg>(Id)` before each heading in an
uncontrolled graphical `ScrollView`. Anchor IDs are unique within the view.
The marker has zero height and takes its place in the containing stack.

`Fpas.Ui.TableOfContents.TableOfContents<TMsg>(Id, ScrollId, Items)` takes one
to 64 entries in document order. Give each entry its title, anchor ID and an
optional indentation level from zero through four. Place the outline in the
same GUI layer as its viewport. CUI handles jumps, tracks scroll progress
and highlights the active entry. Supply `OnActiveChange` to turn changes
to the active anchor into messages. Use `TableOfContents.ItemId(Id, Index)`
to address a row for focus.

Use `FadeEdges := true` on a graphical `ScrollView` to fade the edges with
hidden content. The [Reader page](../../examples/ui/gui/gallery/Pages/Reader/View.fpas)
combines the outline, anchors and edge fades.

## Subview composition

`Fpas.Ui.Composition.Map<TChild, TParent>(Tree, Messages)` embeds a child view
with its own message type. `Messages` is a pure wrapper, typically a case of
the parent's message variant. IDs, options, values, layout and native widget
selection stay unchanged. Every message-bearing node is supported, including
menus, nested overlays, optional callbacks and terminal shortcuts.

Mapping translates immediate message values while building the parent tree.
It wraps callback factories without invoking them. Deferred subviews and terminal
viewport builders remain deferred. Cached subviews retain the original child's
dependency cache; changing the wrapper rebuilds the mapped messages without
rerunning an unchanged child builder. Normal uncached nodes are traversed and
rebuilt; mapping does not provide incremental native layout or virtualization.

Control IDs must be unique across the composed view. The corresponding
[`Fpas.Ui.Mvu.Composition`](../Fpas.Ui.Mvu/README.md#larger-applications) helpers
translate transitions and commands, with optional scopes for background IDs.
Keep cross-page values under one shared owner, as in the
[GUI gallery](../../examples/ui/gui/gallery/README.md#source-structure).

## Host preflight

Call `Validation.Prepare` with the host's profile before presenting any node.
It validates the root, sheet contents and resolved cached
subtrees. It reports an `ArgumentException` with the control ID or tree path
for incompatible units or extensions, unavailable theme selections,
duplicate keys or IDs, and malformed options. Apply any additional
adapter-specific limits before drawing. Draw and input callbacks run at their
respective stages after preflight.
