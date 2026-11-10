# Fpas.Ui.Gui

FPAS desktop adapter for CUI. Applications use [Fpas.Ui.Mvu](../Fpas.Ui.Mvu/README.md)
transitions and [Fpas.Ui](../Fpas.Ui/README.md) declarative views. Start with the
[counter](../../examples/ui/gui/counter/) or explore the [widget gallery](../../examples/ui/gui/gallery/).

## Application lifecycle

`Application.Run` owns a window until native or application close. `Start`
exposes the same host as `Session<TMsg>` for manual owner loops. Both take an
initial model, Update, View and Options. View receives the model and
`Layout.Viewport`; it returns `Elements.Element<TMsg>`. Options select title,
logical size, renderer, mailbox capacity and shared Theme. An optional
`ThemeFor(Model): Fpas.Ui.Themes.Theme` function makes the theme model-owned.
Use the same theme and callback with TUI. CUI palettes, accent families and
customization are described in [shared themes](../Fpas.Ui/README.md#shared-themes).

Create, Step, inspect and dispose a session on one thread. Step presents a frame
and waits for native input when idle. Post admits messages from concurrent
producers, returning the shared Queued/Full/Closed result. Admission and background
completion wake the native event loop. Native input gathered while drawing is
queued after presentation and applied on subsequent owner turns; a temporarily
full mailbox retains those proposals for retry in order without blocking the
window. While proposals are pending the host suppresses further native actions.

The shared StartBackground, CancelBackground, Focus and Exit commands use the
[MVU lifetime protocol](../Fpas.Ui.Mvu/README.md#cancellation-and-shutdown).
Cancellation runs on workers. Closing rejects new messages, discards late results
and continues pumping/painting the frozen view while workers and cancellation
callbacks finish. Resources are released on the owner after cleanup completes.
Disposal joins cleanup while continuing to pump; it is idempotent. Do not reenter
Step or dispose a session from its callbacks. Failures close admission and clean
up before propagating; snapshots retain the last successful presentation.

## Widgets and input

The host implements the graphical profile in the [shared widget reference](../Fpas.Ui/README.md):

- Literal text and separators; all button variants and controlled checkboxes.
- Single-line/multiline Unicode editing with model-owned text and directional
  UTF-16 grapheme selections, distinct edit/navigation messages and placeholders.
- Keyed action/selection lists, tab panels and text tables with column constraints.
  Arrows skip disabled choices; Enter/double-click activation respects CanActivate.
- Percentage progress, rows/columns, content/fill/fixed sizing, bounds, weighted
  growth, automatic content shrinkage, alignment, borders and semantic style roles.
- Vertical clipped scrolling with an optional scrollbar, controlled Start/End/Offset
  and typed user proposals. End follows content growth; layout does not emit input.
- Modal overlays with exclusive input/focus, initial focus (first enabled fallback),
  optional Escape dismissal and restoration of available prior focus.
- Owned packed RGBA images and clipped rectangle/line canvases in logical units.

Visible labels do not encode identity, including literal `##`. Widget IDs and
collection keys are separate, case-sensitive components. Disabled and covered
controls accept no input or focus. Tab/Shift+Tab follow native enabled view order
and stay within the top modal.

Prepare validates the entire active graphical tree before any drawing. Cells and
terminal extensions are rejected with contextual diagnostics. Canvas values must
also fit native floats. Immutable-by-contract records, arrays and image bytes must
remain unchanged while retained by the host. Editor buffers, native control state
and image objects are reconciled by stable identity and released when removed or
replaced; native texture retirement respects frames still using them.

## Invalidation and inspection

Application transitions and observed size changes rebuild the view. Ordinary
repaints reuse its prepared tree and never restart effects. Typed subview caches
advance only after successful presentation. Width classes are Compact below 600,
Regular below 1200, then Expanded; height thresholds are 400 and 900 logical units.
Theme changes preserve model state; include relevant theme dependencies in typed
cached subviews. Platform live-resize callbacks draw only the cached tree without
Update, View, commands or input proposals. Callback state stays rooted until
detachment; managed failures surface after the native pump returns.

Snapshot returns a detached last-successful frame: sequence, logical size, text,
semantic control descriptors (ID, role, item key, value, editor position, enabled,
focus, hover and finalized bounds), resource counts and theme. It is available on
the owner after close/failure; reading before the first presentation throws.
IsSoftware reports actual renderer selection, including fallback.
CapturePng(Path) saves the last presented native frame on the owner outside Step.
It requires a live window, a completed presentation and an existing destination
directory. Capture failures leave the session usable.

## Build and verification

Build examples through the [local SDK workflow](../../docs/build/development.md#build-an-fpas-application).
The source reference delivers CUI's shared library and notices. Applications can
reference the Fpas.Ui.Gui package from the local distribution feed. Its dependencies
deliver Fpas.Ui.Mvu, Fpas.Ui and the [binding package](../Fpas.Ui.Gui.Cui/README.md#packaged-binding)
with prebuilt native assets and static NativeAOT linking. Consumers do not build
CUI. NativeAOT publication needs the .NET platform's native toolchain.
See the [GUI handbook](../../docs/pascal/libraries/gui.md) for the application path,
publishing modes and platform limits.

GuiHostTests builds all three FPAS libraries, the counter and gallery through a
fresh SDK, then exercises CUI input and semantic snapshots under both renderers.
Coverage includes model reconciliation, cache/resize/theme, focus/modal restoration,
background completion, gated worker/cancellation responsiveness, closing redraws,
owner enforcement, failure cleanup and resource removal. CuiHostLayoutTests checks
basis/grow/shrink/bounds; CuiHostResourceTests uses native texture accounting across
replacement and repeated sessions. GuiPublishingTests restores the complete GUI
through packages and exercises the gallery in all five publish modes, including
executable-only NativeAOT and single-file delivery. The
[coverage map](../../tests/coverage.md#native-gui) describes the separate OS
input checks on Windows and Linux.
