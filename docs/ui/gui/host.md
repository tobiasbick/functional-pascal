# Fpas.Ui.Gui

See the [graphical UI handbook](README.md) for illustrated widgets and a first window.

FPAS desktop adapter for CUI. Applications use [Fpas.Ui.Mvu](../mvu/runtime.md)
transitions and [Fpas.Ui](../widgets.md) declarative views. Start with the
[GUI gallery](../../../examples/ui/gui/gallery/README.md).

## Application lifecycle

Import `Fpas.Ui.Gui.Desktop` for window `Options` and the `Run` and `Start` entry
points. Use `Run` for the owner loop or `Start` with `using const` for explicit
`Step` calls. `Fpas.Ui.Gui.Application` defines `Session of TMsg`, `Frame`, and
`Control`.
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
`Continuous` requests frames while idle, for example for video.
`NoVsync` lets OpenGL present without waiting for vertical sync.
`Hidden` creates the window without showing it, for example for off-screen tests.
`ReadingDirection` supplies the direction inherited by the window's widgets.
See [reading direction](layout/reading-direction.md) for container overrides.
All three default to false.

On Windows, CUI temporarily suspends OpenGL vertical sync while the desktop
animates a maximize, minimize or restore command. Presentation returns to the
configured interval after the transition. The bundled Linux host uses X11,
including XWayland on desktops that provide it.

`TraceGeometry.ReadSpanRectangle` reads a trace span's logical rectangle through
an owner-thread MVU command after presentation. The native trace determines
folding, visibility and geometry; a missing span returns an empty rectangle.
See the [native API example](../../../examples/ui/gui/native-api/README.md).

Update and View run on the owner. Shared MVU commands start/cancel background
work, request focus and exit. Close stops admission, cancels work and keeps
pumping the frozen view until cleanup finishes. Dispose joins cleanup. Step,
Snapshot and disposal must not be reentered from callbacks.

Supply `OnCloseRequested` to any Start/Run entry point to turn native closure
into a typed message. Only one decision is pending at a time. Return
`Fpas.Ui.Gui.Closing.Cancel of TMsg()` to reject it and allow another request;
return `Mvu.Exit of TMsg()` to approve, optionally after asynchronous saving.
While a decision is pending, admission and background work remain active.
Without this callback, native close starts cleanup immediately. Explicit
disposal always forces cleanup without invoking the callback. A callback
failure closes resources and propagates through Step. See the runnable
[desktop commands example](../../../examples/ui/gui/desktop-commands/README.md).

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

The first view applies its palette immediately. Later appearance and accent
changes use CUI's palette transition. Zero motion applies the target palette
immediately. The view receives the target theme. A custom canvas can read the
colors of the current transition frame. See [canvas painting](canvas-and-bars/painting.md).

## Fonts and native captions

`Fonts.Apply of TMsg` schedules a typed `Fonts.Operation of TMsg` between frames:
`Load(Bytes, OnLoaded)`, `Assign(Graphics.Font.Body, Face)`, `Fallback(Face, Other)`,
`Emoji(BytesOrNullForSystem, OnLoaded)`, `EmojiFile(Path, OnLoaded)`, and
`SupportsEmoji(Text, OnRead)`.
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
null. `EmojiFile` opens a trusted font file through CUI. CUI can read bitmap
glyphs on demand while the file stays open. Native file opening follows the
platform's path support. Query `SupportsEmoji` for the sequence you need.
A native parse failure can clear the previous emoji resource.
System-font availability and color/monochrome coverage remain platform-dependent.

`Localization.Set of TMsg(CuiStrings)` copies native calendar/file-browser captions.
Month arrays contain twelve entries; weekday arrays seven, Monday first. Null
arrays or entries retain English; a null table restores English. Per-widget
captions take precedence. CUI owns the copied strings until replacement/close.
`Fpas.Ui.Gui.Colors.Oklch` takes a hue in degrees and returns straight RGB
and alpha. `Mix` clamps the amount to zero through one and interpolates with
premultiplied alpha in OKLab. `Alpha` multiplies and clamps the alpha channel
without changing RGB.

See the [painting example](../../../examples/ui/gui/painting/README.md) and
[CUI API coverage](../../cui-api-coverage.md).

## Widgets and input

The [shared reference](../widgets.md) owns options and interaction contracts:

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
Dock panel visibility is learned during presentation. When it changes which
modal is active, the host requests another frame to reconcile input and focus.
The first frame uses the previously observed panel visibility. Place a modal
outside the dockspace when it must block all other controls immediately.

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

- `OpenFile of TMsg(OnSelected, Title, FilterName, Accept, OnCancelled)` selects
  an existing file and checks the selected path against `Accept`.
- `SaveFile of TMsg(OnSelected, DefaultName, Title, FilterName, Accept, OnCancelled)`
  selects a destination without creating or writing it. `DefaultName` can be
  a filename or path. The native dialog owns filter and overwrite behavior.
- `ChooseFolder of TMsg(OnSelected, Title, OnCancelled)` selects a directory.

Only `OnSelected` is required. `Accept` lists extensions without dots,
separated by semicolons; empty accepts any type. The owner opens CUI's platform
dialog between frames. It is modal: Update/View pause while the dialog is open,
but background runs continue. A selected Unicode path becomes a typed message.
Optional `OnCancelled` emits a message when CUI returns no selection; otherwise
the update receives nothing. CUI also returns no selection when the platform
helper is unavailable or Open returns a filter mismatch. Linux requires
Zenity or KDialog on `PATH`. Dialog messages map through MVU feature slices.

To ask the desktop to open a trusted URL or absolute path with its registered
handler, call `Fpas.Ui.Gui.Desktop.TryOpen(Target)`. It returns whether CUI
could dispatch the request. On Linux this requires `xdg-open`, and the
handler may finish asynchronously.

## Clipboard

Return `Fpas.Ui.Gui.Clipboard.WriteText of TMsg(Text)` to copy plain text from an
update, such as a menu command that copies a path. The command submits Unicode
text to the clipboard provider of the owner window. Text can contain line
breaks and must contain no NUL characters. Return
`Fpas.Ui.Gui.Clipboard.ReadText of TMsg(OnRead)` to read text. The host applies
`OnRead` to the clipboard text on its owner thread and queues the resulting
message. Unavailable clipboard text is passed as an empty string.

## Application shortcuts

Supply `BindingsFor(Model)` to Start/Run or their themed variants. It returns
immutable `Fpas.Ui.Gui.Shortcuts.Binding of TMsg` values using the existing
`ShortcutRecorders.Combination`, `Key`, `Modifier` and `PrimaryModifier()`.
Each binding carries its message and the Enabled, Repeat, AllowWhileEditing, and AllowInOverlays policies.
`Binding.Map of TParent(Messages)` wraps a feature slice's message. Bindings are
prepared with the model view and reused on native repaints; enabled key/modifier
combinations must be unique. None is not an application key.

CUI processes widgets first. By default, popups and modals block application
bindings through their dismissal frame. Use `AllowInOverlays` for an
unconsumed combination such as changing the gallery appearance.
Focused widgets retain navigation/activation
and bare typing keys. By default, text editing blocks application bindings;
`AllowWhileEditing` explicitly enables global commands such as Save. Use it
only for combinations not assigned to native editing; already-consumed keys
remain unavailable. Matching requires the exact modifier combination.
Repeat is off by default and requires a release observed by the native frame
loop before another activation. With Repeat enabled, CUI's OS repeats may
activate once per frame. Disabled bindings do not claim input or reactivate a
held key on enable. Closing admits no shortcut messages.

See the runnable [editor commands example](../../../examples/ui/gui/editor-commands/README.md).

## Code editor commands

Return commands from `Fpas.Ui.Gui.EditorCommands` in an update:

- `GoTo of TMsg(Id, Line, Column)` focuses/scrolls to a one-based native line and
  visual column, including tab stops. CUI clamps beyond the document.
- `Select of TMsg(Id, Anchor, Cursor)` focuses and selects a range in the next
  view's model text. Offsets are UTF-16 code units at Unicode scalar boundaries;
  reversed ranges are valid and invalid positions are rejected.
- `Find of TMsg(Id, Query)` opens CUI's find bar and selects the next match from
  the current caret/selection. Null closes the bar and restores editor focus.
  A query holds at most 255 UTF-8 bytes; longer or malformed text is rejected.

Only the last request per editor ID is retained for the next normal view.
Absent or disabled editors discard their requests. Cached live-resize painting
does not consume them; closing/discarding a view clears pending requests.
Read-only editors still permit navigation, selection and finding. Commands
map through MVU feature slices without a second editor implementation.
CUI owns text editing, search, scrolling, highlighting and deferred paint.
The shared [editor reference](../widgets.md#native-source-tools) owns
observations and highlight options.

## Window state and placement

Return a command from `Fpas.Ui.Gui.Windows` to change the owner window:

- `SetTitle of TMsg(Title)` updates the native document title.
- `Resize of TMsg(Width, Height)` requests positive logical dimensions.
- `Maximize of TMsg()` and `Minimize of TMsg()` change its state where the window
  manager supports it.
- `Restore of TMsg()` undoes a maximize or a minimize. A window minimized from
  maximized comes back maximized.
- `MoveTo of TMsg(X, Y)` places the top-left corner of the content.
- `ReadPlacement of TMsg(OnRead)` applies `OnRead` to the window's
  `Cui.CuiWindowPlacement` on the owner thread and queues the message.

A placement holds the normal size and place, also while the window is
maximized or minimized, and whether the window is or comes back maximized.
Save it and pass it as `Options.Placement` at the next start. Its size
replaces `Width` and `Height` and is clamped to CUI's minimum window size.
Read the final placement for saving from the session: drive `Start` or `StartThemed` with `Step` and
call `Session.Placement()` after the last step and before `Dispose`. It then
returns the placement the window had when it closed.

`Session.InspectWindow()` reads live logical size, current state, backend
name and native handle on the owner thread. A minimized window reports its
current state here and its restore state in the placement. Use the backend
name to interpret the borrowed handle, such as an HWND on Win32 or an X11
Window on X11. The handle belongs to the session while the window is live.
The same result reports DPI, focus, file drag and unclaimed drop state.
[Window observation](app-chrome/window.md#window-observation)

The position is in desktop coordinates: pixels on Windows and X11, points on
macOS. Keep it only to pass it back. A place on no screen is left to the
system. Wayland reports `HasPosition = false` and ignores moves. On X11 the
window manager carries out state changes, so a new state can take a few frames
to show. A window manager without EWMH support ignores them.

## Toasts

Return `Fpas.Ui.Gui.Toasts.Show of TMsg` as a command from an update. Each
command queues one native notification after the model is committed.
`Options` sets the description, tone, duration, action caption and draining
timer. CUI defaults to four seconds, or six seconds with the timer line.
A negative duration keeps the toast until dismissal. Hovering expands the
stack and pauses the countdown. Close or swipe right to dismiss it.

An action caption adds a button that dismisses the toast. If you supply an
`OnAction` message factory, the host calls it once when the user activates the
button.
The action can be reached by Tab and activated by Enter or Space.
Toasts leave the current focus in place when they appear. The
`Snapshot().Controls` list includes visible cards and buttons.
The terminal host rejects this graphical command.

Title, description and action caption are limited to 127, 191 and 47 UTF-8
bytes respectively. The native action caption reserves `##` as a label
delimiter.

## Invalidation and inspection

Return `Fpas.Ui.Gui.Frames.Redraw of TMsg(DelaySeconds)` to request a native
frame after the current presentation. Zero requests the next frame; a finite
positive value supplies seconds until a wakeup. CUI uses the earliest deadline
and can render sooner for input or animation. This requests a repaint, emits
no message and does not rebuild the cached view or create a managed timer.
Use background commands and typed messages for timed model updates. Pending
frame requests are discarded at close; normal idle waiting remains enabled.

Transitions, viewport size changes and observed system appearance changes
rebuild the view. Ordinary repaints reuse its prepared tree and never restart
effects. Typed caches advance after successful presentation. The size classes
and `LogicalWidth` and `LogicalHeight` passed to `View` come from the CUI
frame's drawable area. `Snapshot().Size` reports the dimensions of the last
presented frame. Include relevant theme dependencies in cached subviews.
[Responsive views](../layout/responsive.md) lists the size class thresholds.

During the native Windows resize loop, CUI lays out the cached view at the
new size. The regular session step rebuilds `View` and publishes its snapshot.

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

## Frame preparation and presentation

The session prepares the shared element tree, checks graphical layout rules,
and gives the prepared tree to the presentation pass. Presentation converts
FPAS options and values to CUI data, sets native layout hints, and calls each
widget adapter. Native changes become proposals that the session can deliver
to the model. The frame context records control IDs and their native tokens.
After CUI resolves layout, the host uses those tokens to attach finalized
bounds to control descriptors for inspection.

Focus resolution walks the prepared tree to find enabled targets, including
keyed parts inside composite widgets. The host tracks the active modal and
remembers the preceding focus target for restoration when it closes.

## Build and verification

See [development](../../build/development.md), the
[native prerequisites](native-build.md#prerequisites) and the
[binding verification](cui.md#source-build-and-verification).
The GUI gallery contract exercises both renderers. Set `FPAS_TEST_GUI_AOT=1`
to include the publishing tests for all five modes, including single-file and
Native AOT. Linux window tests require a display.

The [CUI inventory](../../cui-widget-inventory.md#acceptance) records GUI
gallery acceptance and existing TUI coverage.

## Source files

- [CUI capability contract](../../../tests/Fixtures/gui-host/CuiCapabilitiesContract.fpas)

<details>
<summary>Browse 62 source files</summary>

- `examples/ui/gui/gallery/Pages/Desktop/`
  - [Editor.fpas](../../../examples/ui/gui/gallery/Pages/Desktop/Editor.fpas)
  - [HostActions.fpas](../../../examples/ui/gui/gallery/Pages/Desktop/HostActions.fpas)
- `src/Fpas.Ui.Gui/`
  - [Application.fpas](../../../src/Fpas.Ui.Gui/Application.fpas)
  - [Clipboard.fpas](../../../src/Fpas.Ui.Gui/Clipboard.fpas)
  - [Closing.fpas](../../../src/Fpas.Ui.Gui/Closing.fpas)
  - [Colors.fpas](../../../src/Fpas.Ui.Gui/Colors.fpas)
  - [Desktop.fpas](../../../src/Fpas.Ui.Gui/Desktop.fpas)
  - [EditorCommands.fpas](../../../src/Fpas.Ui.Gui/EditorCommands.fpas)
  - [FileDialogs.fpas](../../../src/Fpas.Ui.Gui/FileDialogs.fpas)
  - [Fonts.fpas](../../../src/Fpas.Ui.Gui/Fonts.fpas)
  - [Fpas.Ui.Gui.fpasproj](../../../src/Fpas.Ui.Gui/Fpas.Ui.Gui.fpasproj)
  - [Frames.fpas](../../../src/Fpas.Ui.Gui/Frames.fpas)
  - [Localization.fpas](../../../src/Fpas.Ui.Gui/Localization.fpas)
  - [Shortcuts.fpas](../../../src/Fpas.Ui.Gui/Shortcuts.fpas)
  - [Toasts.fpas](../../../src/Fpas.Ui.Gui/Toasts.fpas)
  - [Windows.fpas](../../../src/Fpas.Ui.Gui/Windows.fpas)
- `src/Fpas.Ui.Gui/Internal/Core/`
  - [Context.fpas](../../../src/Fpas.Ui.Gui/Internal/Core/Context.fpas)
  - [Parts.fpas](../../../src/Fpas.Ui.Gui/Internal/Core/Parts.fpas)
  - [Values.fpas](../../../src/Fpas.Ui.Gui/Internal/Core/Values.fpas)
- `src/Fpas.Ui.Gui/Internal/Data/`
  - [AiData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/AiData.fpas)
  - [CanvasData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/CanvasData.fpas)
  - [ChartData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/ChartData.fpas)
  - [CurveData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/CurveData.fpas)
  - [FileData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/FileData.fpas)
  - [MediaData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/MediaData.fpas)
  - [MoreChartData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/MoreChartData.fpas)
  - [MotionData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/MotionData.fpas)
  - [ScheduleData.fpas](../../../src/Fpas.Ui.Gui/Internal/Data/ScheduleData.fpas)
- `src/Fpas.Ui.Gui/Internal/Hosting/`
  - [FontRequests.fpas](../../../src/Fpas.Ui.Gui/Internal/Hosting/FontRequests.fpas)
  - [FrameControls.fpas](../../../src/Fpas.Ui.Gui/Internal/Hosting/FrameControls.fpas)
  - [KeyBindings.fpas](../../../src/Fpas.Ui.Gui/Internal/Hosting/KeyBindings.fpas)
  - [Requests.fpas](../../../src/Fpas.Ui.Gui/Internal/Hosting/Requests.fpas)
  - [Session.fpas](../../../src/Fpas.Ui.Gui/Internal/Hosting/Session.fpas)
  - [ThemeApplication.fpas](../../../src/Fpas.Ui.Gui/Internal/Hosting/ThemeApplication.fpas)
- `src/Fpas.Ui.Gui/Internal/Input/`
  - [AiFocus.fpas](../../../src/Fpas.Ui.Gui/Internal/Input/AiFocus.fpas)
  - [CreativeFocus.fpas](../../../src/Fpas.Ui.Gui/Internal/Input/CreativeFocus.fpas)
  - [Editor.fpas](../../../src/Fpas.Ui.Gui/Internal/Input/Editor.fpas)
  - [Focus.fpas](../../../src/Fpas.Ui.Gui/Internal/Input/Focus.fpas)
  - [Modal.fpas](../../../src/Fpas.Ui.Gui/Internal/Input/Modal.fpas)
  - [ModalFocus.fpas](../../../src/Fpas.Ui.Gui/Internal/Input/ModalFocus.fpas)
- `src/Fpas.Ui.Gui/Internal/Layout/`
  - [LayoutValidation.fpas](../../../src/Fpas.Ui.Gui/Internal/Layout/LayoutValidation.fpas)
  - [RealizedValidation.fpas](../../../src/Fpas.Ui.Gui/Internal/Layout/RealizedValidation.fpas)
  - [Sizing.fpas](../../../src/Fpas.Ui.Gui/Internal/Layout/Sizing.fpas)
- `src/Fpas.Ui.Gui/Internal/Presentation/`
  - [CanvasPresentation.fpas](../../../src/Fpas.Ui.Gui/Internal/Presentation/CanvasPresentation.fpas)
  - [ChromePresentation.fpas](../../../src/Fpas.Ui.Gui/Internal/Presentation/ChromePresentation.fpas)
  - [DeveloperPresentation.fpas](../../../src/Fpas.Ui.Gui/Internal/Presentation/DeveloperPresentation.fpas)
  - [DisplayPresentation.fpas](../../../src/Fpas.Ui.Gui/Internal/Presentation/DisplayPresentation.fpas)
  - [EditingToolPresentation.fpas](../../../src/Fpas.Ui.Gui/Internal/Presentation/EditingToolPresentation.fpas)
  - [Presentation.fpas](../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
  - [SourceToolPresentation.fpas](../../../src/Fpas.Ui.Gui/Internal/Presentation/SourceToolPresentation.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.Toast.cs](../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.Toast.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.Toast.Types.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Toast.Types.cs)
  - [CuiWindow.Toast.cs](../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Toast.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [toast.c](../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/toast.c)
  - [toast.h](../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/toast.h)
- `tests/Fixtures/gui-host/`
  - [DesktopSchedulingContract.fpas](../../../tests/Fixtures/gui-host/DesktopSchedulingContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Layout/`
  - [CuiHostLayoutTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiHostLayoutTests.cs)
- `tests/Fpas.Ui.Gui.Cui.Tests/Packaging/`
  - [GuiHostTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/GuiHostTests.cs)
- `tests/Fpas.Ui.Gui.Cui.Tests/Resources/`
  - [CuiFontThemeTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Resources/CuiFontThemeTests.cs)
- `tests/Fpas.Ui.Gui.Cui.Tests/Windowing/`
  - [CuiFrameSchedulingTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiFrameSchedulingTests.cs)
  - [CuiHostLifecycleReviewTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiHostLifecycleReviewTests.cs)
  - [CuiWakeupTests.cs](../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiWakeupTests.cs)

</details>
