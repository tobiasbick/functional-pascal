# Fpas.Ui.Gui

FPAS desktop adapter for CUI. Applications use [Fpas.Ui.Mvu](../Fpas.Ui.Mvu/README.md)
transitions and [Fpas.Ui](../Fpas.Ui/README.md) declarative views. Start with the
[GUI gallery](../../examples/ui/gui/gallery/README.md).

## Application lifecycle

Use Run for the owner loop or Start with using const for explicit Step calls.
Create, step, inspect and dispose sessions on their creating UI thread. Post
admits typed messages from workers and reports Queued, Full or Closed.
Use `Options` to set the title, logical width and height, renderer, mailbox
capacity and theme. The initial width and height supply the normal window size.
`MinimumWidth` and `MinimumHeight` set native minimum dimensions; zero uses
CUI's defaults. `FixedSize` disables interactive resizing/maximization.
`AppId` supplies the Linux desktop identity, matching the installed `.desktop`
filename without its suffix; Windows ignores it. These are creation options.
Set `Placement` to open at a saved place, maximized if it was; see
[window state and placement](#window-state-and-placement).

`TraceGeometry.ReadSpanRectangle` reads a trace span's logical rectangle through
an owner-thread MVU command after presentation. The native trace determines
folding, visibility and geometry; a missing span returns an empty rectangle.
See the [native API example](../../examples/ui/gui/native-api/README.md).

Update and View run on the owner. Shared MVU commands start/cancel background
work, request focus and exit. Close stops admission, cancels work and keeps
pumping the frozen view until cleanup finishes. Dispose joins cleanup. Step,
Snapshot and disposal must not be reentered from callbacks.

Supply `OnCloseRequested` to any Start/Run entry point to turn native closure
into a typed message. Only one decision is pending at a time. Return
`Fpas.Ui.Gui.Closing.Cancel<TMsg>()` to reject it and allow another request;
return `Mvu.Exit<TMsg>()` to approve, optionally after asynchronous saving.
While a decision is pending, admission and background work remain active.
Without this callback, native close starts cleanup immediately. Explicit
disposal always forces cleanup without invoking the callback. A callback
failure closes resources and propagates through Step. See the runnable
[desktop commands example](../../examples/ui/gui/desktop-commands/README.md).

## System appearance

`StartThemed` and `RunThemed` pass the CUI-reported light or dark mode to
`ThemeFor(Model, SystemAppearance)`. The mode is `null` when CUI has no value.
Select the palette using the application's appearance preference and accent.
The host applies the returned theme to CUI and passes it to the declarative
view. Windows appearance changes wake the window and rebuild the view without
an application message. CUI's X11 backend can report a dark `GTK_THEME` hint.
It supplies no live desktop appearance notification.

The themed entry points enable CUI's native appearance following. On a native
change the host resolves the model's theme again, including custom palette
overrides and `Theme.Metrics`; the native theme never becomes a second model.

## Fonts and native captions

`Fonts.Apply<TMsg>` schedules a typed `Fonts.Operation<TMsg>` between frames:
`Load(Bytes, OnLoaded)`, `Assign(Graphics.Font.Body, Face)`, `Fallback(Face, Other)`,
`Emoji(BytesOrNullForSystem, OnLoaded)` and `SupportsEmoji(Text, OnRead)`.
Use CLR file APIs to obtain trusted font bytes. Loading copies them; completion
returns a `CuiFontFace` or null. Retain and reuse loaded faces in the model.
Faces belong to one window and cannot cross contexts or threads.

`Assign` with null restores the built-in role. CUI has no individual font-unload
API: replacing a role does not free the old face, and window disposal frees
every loaded face. A face supports four direct fallbacks; duplicates and
self-fallbacks are rejected. Direct fallbacks precede CUI's automatic search
among loaded faces and the icon role. Icons require a face dedicated to that role.
Changes occur between frames, so deferred painting cannot outlive its resources.

Emoji replacement owns its bytes, or asks CUI for a system font when data is
null. Acceptance does not promise glyph coverage; query `SupportsEmoji` for the
sequence needed. A native parse failure can clear the previous emoji resource.
System-font availability and color/monochrome coverage remain platform-dependent.

`Localization.Set<TMsg>(CuiStrings)` copies native calendar/file-browser captions.
Month arrays contain twelve entries; weekday arrays seven, Monday first. Null
arrays or entries retain English; a null table restores English. Per-widget
captions take precedence. CUI owns the copied strings until replacement/close.
`Fpas.Ui.Gui.Colors` supplies native OKLCH, OKLab mixing and alpha helpers.

See the [painting example](../../examples/ui/gui/painting/README.md) and
[CUI API coverage](../../docs/cui-api-coverage.md).

## Widgets and input

The [shared reference](../Fpas.Ui/README.md) owns options and interaction contracts:

- Buttons, icon buttons and connected button groups.
- Copy, hold, confirmation, status and split buttons.
- Theme switcher and sun/moon toggle with model-owned palette updates.
- Segmented controls with labels/icons, native sizes and full-width allocation.
- Toggle groups with independent or exclusive selection, labels and icon tooltips.
- Checkbox, switch and radio rows with descriptions and controlled selection.
- Responsive radio cards with icons, descriptions and badges.
- Continuous and stepped sliders, and a range slider with two draggable endpoints.
- Selection chips and whole-star ratings.
- Dropdown menus with native triggers, command states, hints, labels and separators.
- Context menus attached to existing target components, using the same menu entries.
- Reactions with model-owned counts and native animation, and native account menus.
- Speed dial with native floating actions and swipe rows with pointer and horizontal wheel input.
- Anchored popovers with model-owned visibility and native confirmation questions.
- Native hover tooltips with optional shortcut key hints.
- Single-line text fields with native editing, validation text, affixes and password reveal.
- Search fields with a native clear action and text areas with wrapping and wheel scrolling.
- Tag inputs, one-time code boxes and native password strength meters.
- Plain and searchable select lists and bounded number fields with native step buttons.
- Searchable multi-select with native checkboxes and removable selection chips.
- Native date range pickers with optional presets and inline date range calendars.
- Native single-date fields and inline calendars with month navigation.
- Native RGB palettes and searchable emoji pickers.
- Native time fields, selection wheels with joined bands, and an analog time dial.
- Native measurement rulers and adjustable ranges over histogram data.
- Native inline links, avatars with images or initials, and pointer-driven hover cards.
- Native announcement banners, rolling countdowns and onboarding checklists.
- Native file dropzones, uploadable avatars and signature pads with Clear.
- Native audio knobs, faders with channel meters, and standalone peak and clip meters.
- Sidebar navigation and edge sheets/drawers.
- Native responsive grids, main-axis justification, cards, badges, metrics,
  overlapping avatar groups and empty-state actions.
- Native tables of contents and scroll anchors. Snapshots expose the active
  anchor, the marked row and the jump destination bounds.
- Centered modal dialogs with native action footers and focus return.
- Underline and pill tabs, breadcrumb links, responsive pagination and steppers.
- Native toasts with stacking, hover pause and an optional timed action.
- Text, separators, wrapping rows, columns, sizing, borders, semantic styles and scrolling
  as the foundations needed to compose these widgets.

Visible labels and stable IDs are separate, including literal ## text. Disabled
and covered controls accept no input or focus. Tab/Shift+Tab traverse enabled
controls within the active modal. When a modal closes, the host tries to
return focus to the last control focused before it opened. If that control
is unavailable, focus goes to the first enabled control in the presented
scope, when available. Native CUI supplies rendering and interaction.
The host maps proposals to messages without applying application updates itself.

Prepare validates the active view before drawing. GUI distances use logical
units; terminal-specific elements and cell distances are rejected. View records,
arrays and callbacks remain unchanged while retained by the host.
Place modal scopes outside native-owned collapsible Reasoning bodies; validation
rejects that composition even while the modal is closed. In virtual lists and
data grids, declare open modal scopes outside row/cell factories. Only realized
subtrees are checked, before drawing their modal contents.

A visible native tooltip appears in `Snapshot().Controls` with role `tooltip`
and its finalized bounds. The underlying control retains its own identity.

## File selection

Return a command from `Fpas.Ui.Gui.FileDialogs` in the update:

- `OpenFile<TMsg>(OnSelected, Title, FilterName, Accept, OnCancelled)` selects
  an existing file and checks the selected path against `Accept`.
- `SaveFile<TMsg>(OnSelected, DefaultName, Title, FilterName, Accept, OnCancelled)`
  selects a destination without creating or writing it. `DefaultName` can be
  a filename or path. The native dialog owns filter and overwrite behavior.
- `ChooseFolder<TMsg>(OnSelected, Title, OnCancelled)` selects a directory.

Only `OnSelected` is required. `Accept` lists extensions without dots,
separated by semicolons; empty accepts any type. The owner opens CUI's platform
dialog between frames. It is modal: Update/View pause while the dialog is open,
but background runs continue. A selected Unicode path becomes a typed message.
Optional `OnCancelled` emits a message when CUI returns no selection; otherwise
the update receives nothing. CUI also returns no selection when the platform
helper is unavailable or Open returns a filter mismatch. Linux requires
Zenity or KDialog on `PATH`. Dialog messages map through MVU feature slices.

## Clipboard

Return `Fpas.Ui.Gui.Clipboard.WriteText<TMsg>(Text)` to copy plain text from an
update, such as a menu command that copies a path. Return
`Fpas.Ui.Gui.Clipboard.ReadText<TMsg>(OnRead)` to read text. The host applies
`OnRead` to the clipboard text on its owner thread and queues the resulting
message. Unavailable clipboard text is passed as an empty string.

## Application shortcuts

Supply `BindingsFor(Model)` to Start/Run or their themed variants. It returns
immutable `Fpas.Ui.Gui.Shortcuts.Binding<TMsg>` values using the existing
`ShortcutRecorders.Combination`, `Key`, `Modifier` and `PrimaryModifier()`.
Each binding carries its message, Enabled, Repeat and AllowWhileEditing policy.
`Binding.Map<TParent>(Messages)` wraps a feature slice's message. Bindings are
prepared with the model view and reused on native repaints; enabled key/modifier
combinations must be unique. None is not an application key.

CUI processes widgets first. Open popups and modals block application bindings,
including their dismissal frame. Focused widgets retain navigation/activation
and bare typing keys. By default, text editing blocks application bindings;
`AllowWhileEditing` explicitly enables global commands such as Save. Use it
only for combinations not assigned to native editing; already-consumed keys
remain unavailable. Matching requires the exact modifier combination.
Repeat is off by default and requires a release observed by the native frame
loop before another activation. With Repeat enabled, CUI's OS repeats may
activate once per frame. Disabled bindings do not claim input or reactivate a
held key on enable. Closing admits no shortcut messages.

See the runnable [editor commands example](../../examples/ui/gui/editor-commands/README.md).

## Code editor commands

Return commands from `Fpas.Ui.Gui.EditorCommands` in an update:

- `GoTo<TMsg>(Id, Line, Column)` focuses/scrolls to a one-based native line and
  visual column, including tab stops. CUI clamps beyond the document.
- `Select<TMsg>(Id, Anchor, Cursor)` focuses and selects a range in the next
  view's model text. Offsets are UTF-16 code units at Unicode scalar boundaries;
  reversed ranges are valid and invalid positions are rejected.
- `Find<TMsg>(Id, Query)` opens CUI's find bar and selects the next match from
  the current caret/selection. Null closes the bar and restores editor focus.
  A query holds at most 255 UTF-8 bytes; longer or malformed text is rejected.

Only the last request per editor ID is retained for the next normal view.
Absent or disabled editors discard their requests. Cached live-resize painting
does not consume them; closing/discarding a view clears pending requests.
Read-only editors still permit navigation, selection and finding. Commands
map through MVU feature slices without a second editor implementation.
CUI owns text editing, search, scrolling, highlighting and deferred paint.
The shared [editor reference](../Fpas.Ui/README.md#native-source-tools) owns
observations and highlight options.

## Window state and placement

Return a command from `Fpas.Ui.Gui.Windows` to change the owner window:

- `SetTitle<TMsg>(Title)` updates the native document title.
- `Resize<TMsg>(Width, Height)` requests positive logical dimensions.
- `Maximize<TMsg>()` and `Minimize<TMsg>()` change its state where the window
  manager supports it.
- `Restore<TMsg>()` undoes a maximize or a minimize. A window minimized from
  maximized comes back maximized.
- `MoveTo<TMsg>(X, Y)` places the top-left corner of the content.
- `ReadPlacement<TMsg>(OnRead)` applies `OnRead` to the window's
  `Cui.CuiWindowPlacement` on the owner thread and queues the message.

A placement holds the normal size and place, also while the window is
maximized or minimized, and whether the window is or comes back maximized.
Save it and pass it as `Options.Placement` at the next start. Its size
replaces `Width` and `Height` and is clamped to CUI's minimum window size.
Read the final placement for saving from the session: drive `Start` or `StartThemed` with `Step` and
call `Session.Placement()` after the last step and before `Dispose`. It then
returns the placement the window had when it closed.

The position is in desktop coordinates: pixels on Windows and X11, points on
macOS. Keep it only to pass it back. A place on no screen is left to the
system. Wayland reports `HasPosition = false` and ignores moves. On X11 the
window manager carries out state changes, so a new state can take a few frames
to show. A window manager without EWMH support ignores them.

## Toasts

Return `Fpas.Ui.Gui.Toasts.Show<TMsg>` as a command from an update. Each
command queues one native notification after the model is committed.
`Options` sets the description, tone, duration, action caption and draining
timer. CUI defaults to four seconds, or six seconds with the timer line.
A negative duration keeps the toast until dismissal. Hovering expands the
stack and pauses the countdown. Close or swipe right to dismiss it.

Supply an action caption and an `OnAction` message factory together.
The host invokes the factory once after the native action is activated.
The action can be reached by Tab and activated by Enter or Space.
Toasts leave the current focus in place when they appear. The
`Snapshot().Controls` list includes visible cards and buttons.
The terminal host rejects this graphical command.

Title, description and action caption are limited to 127, 191 and 47 UTF-8
bytes respectively. The native action caption reserves `##` as a label
delimiter.

## Invalidation and inspection

Return `Fpas.Ui.Gui.Frames.Redraw<TMsg>(DelaySeconds)` to request a native
frame after the current presentation. Zero requests the next frame; a finite
positive value supplies seconds until a wakeup. CUI uses the earliest deadline
and can render sooner for input or animation. This requests a repaint, emits
no message and does not rebuild the cached view or create a managed timer.
Use background commands and typed messages for timed model updates. Pending
frame requests are discarded at close; normal idle waiting remains enabled.

Transitions, size changes and observed system appearance changes rebuild the
view. Ordinary repaints reuse its
prepared tree and never restart effects. Typed caches advance after successful
presentation. Width classes are Compact below 600, Regular below 1200, then
Expanded; height thresholds are 400 and 900 logical units. Include relevant
theme dependencies in cached subviews.

Platform live-resize callbacks draw the cached tree without Update, View,
commands or input proposals. Callback state stays rooted until detachment;
managed failures surface after the native pump returns.
Typewriter completion factories run once on a normal owner frame, including
when covered by a modal. Completion during a cached resize is delivered on
the next owner frame; closing painting invokes no completion factories.

Snapshot returns the last successful frame: sequence, logical size, text, theme
and detached control descriptors with ID, role, item key, value, enabled state,
focus, hover and bounds. Reading before the first presentation throws.
IsSoftware reports the actual renderer, including fallback.
CapturePng saves the presented frame outside Step on the owner thread. It needs
a live window, a successful presentation and an existing destination directory.

## Build and verification

See [development](../../docs/build/development.md), the
[native prerequisites](../Fpas.Ui.Gui.Cui/native/README.md#prerequisites) and the
[binding verification](../Fpas.Ui.Gui.Cui/README.md#source-build-and-verification).
The GUI gallery contract exercises both renderers. Set `FPAS_TEST_GUI_AOT=1`
to include the publishing tests for all five modes, including single-file and
Native AOT. Linux window tests require a display.

The [CUI inventory](../../docs/cui-widget-inventory.md#acceptance) records GUI
gallery acceptance and existing TUI coverage.
