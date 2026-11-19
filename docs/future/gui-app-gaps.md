# Desktop GUI application gaps

The first target is a small FPAS Notes app that opens and saves files, handles
keyboard shortcuts and asks about unsaved changes before closing. Use the
[GUI handbook](../ui/gui/desktop.md) for the supported API and the
[CUI widget inventory](../cui-widget-inventory.md) for gallery acceptance. Keep CUI responsible
for native rendering and input. Use FPAS MVU for application state, commands
and typed messages. The acceptance targets are Windows x64 and Linux x64.

The source review covers CUI's [Building an app][apps], [Recipes][recipes],
[Getting started][getting-started] and [Theming][theming] at the
[vendored revision](../../third-party/cui/UPSTREAM.md).

## First app and tutorial

- [ ] Add a standalone Notes example and a step-by-step FPAS tutorial. Build a
  list and detail pane with the existing
  [SplitView](../../src/Fpas.Ui/Widgets/Desktop/SplitViews.fpas). Add a form
  that shows errors from the first submit attempt onward. Route Enter in the
  last field and the submit button to the same update. Use the existing
  [text-field error and submit options](../../src/Fpas.Ui/Widgets/Inputs/TextFields.fpas).
  The gallery workspace form illustrates the layout and currently shows a
  success toast without validating the fields.
- [ ] Show how to sort a table from its header proposals. Keep selection by
  stable record key across a sort. Use the existing
  [table API](../ui/widgets.md#data-display) and show when to use a
  [data-grid row factory](../../src/Fpas.Ui/Widgets/Collections/DataGrids.fpas)
  for visible rows in large lists.
- [ ] Restore the saved appearance, accent and split ratio before the first
  frame. Show how to read and write application settings with .NET I/O.
  Keep defaults for missing or invalid settings and handle write failures
  through an application message. Dock layouts can use the existing
  [Dockspace layout callback](../../src/Fpas.Ui/Widgets/Desktop/Dockspaces.fpas).
- [ ] Add short, runnable examples for background progress and time-driven
  updates. Use [StartBackground and typed Post](../ui/mvu/runtime.md#background-effects)
  for progress, errors and timer ticks. Demonstrate cancellation and updates
  without user input. The
  [gallery countdown](../../examples/ui/gui/gallery/Pages/Disclosure/Clock.fpas)
  already uses this pattern. For repaints without model changes, use
  [native frame scheduling](../ui/gui/host.md#invalidation-and-inspection).

Use the Notes app to verify opening, editing, saving, reopening and restoring
settings. Cover button and keyboard submission, selection after sorting, and
both ways to handle unsaved changes. Keep the examples small enough to build
through the documented SDK workflow.

## Document commands and window close

The [desktop command API](../ui/gui/host.md#file-selection) and
[runnable example](../../examples/ui/gui/desktop-commands/README.md) supply the
native operations. The Notes application must exercise Open/Save/Choose Folder with Unicode
paths, filters and cancellation, then Save As and restoring the resulting title.
Verify a clean close, dirty close, repeated close requests and failed/cancelled
saves. Only an approved close stops and joins background work; opt into the
GUI host's `OnCloseRequested` callback and reject with `Closing.Cancel` or
approve with `Mvu.Exit`. Exercise model-owned
[application bindings](../ui/gui/host.md#application-shortcuts)
with primary modifiers, single activation and native overlay/editing ownership.
Use AllowWhileEditing only for global commands such as Save.
[CUI file dialogs][file-dialogs] · [CUI keyboard shortcuts][shortcuts]

- [ ] Run the focused desktop command acceptance on Windows: native Open,
  Save and Choose Folder, Unicode/default names/filters/cancellation, repeated
  close decisions, worker cleanup, title/size options and idle frame deadlines.
  Include application shortcuts and native editor find/navigation/Unicode
  selection and focus return on Windows.
  Linux X11 automation does not establish Windows behavior.

## Desktop publishing and window configuration

- [ ] Support the Windows executable's application icon and version metadata
  and document how to provide them in the project.
  [CUI shipping guide][shipping]

Use the supported [window options and commands](../ui/gui/host.md#window-state-and-placement).
For Linux delivery, match application identity to the installed `.desktop`
filename/icon and check X11 `WM_CLASS` and desktop grouping. CUI reads the ID at
creation and ignores it on Windows. Verify restoring saved dimensions and
changing a document title in Notes.
[CUI desktop identity][desktop-id] · [CUI window API][window-api]

## Appearance and accessibility

Use the [font and native caption commands](../ui/gui/host.md#fonts-and-native-captions)
and shared theme metrics for platform acceptance.
Platform acceptance must cover a running animation and visible CJK glyphs with
an explicit fallback. Linux live appearance needs change notifications for an
idle window; CUI's X11 backend currently supplies a dark `GTK_THEME` hint.
Font coverage and bidirectional editing are separate concerns.
[CUI app options][app-options] · [CUI typography and motion][theming]

- [ ] Establish a screen-reader plan before treating the GUI as accessible for
  public apps. CUI has no accessibility tree. FPAS snapshots expose bounds,
  roles and values for inspection. A screen reader also needs platform
  semantics, focus events and actions. Start by validating a form, a modal and
  a list with a real screen reader. Track IME and complex-script editing
  separately. [CUI limitations][limitations] ·
  [FPAS platform limits](../ui/gui/desktop.md#publishing-and-platform-limits)

## Widget documentation captures

- [ ] Crop the widget documentation captures in FPAS. The current capture
  project drives the real gallery, writes full PNGs through `Session.CapturePng`
  and records crop bounds from the presented snapshot. It uses Pillow for
  decoding, cropping and writing those PNGs because the shared .NET runtime
  and the FPAS GUI API have no portable bitmap crop operation. Verify the
  replacement against the same scale-one frame and crop bounds.
  [Capture workflow](../../apps/gui-doc-captures/README.md)

## Deferred window chrome

The [optional title bar](cui-titlebar.md) has a separate plan. The local CUI
prototype at `00eddec` adds an experimental title bar on Win32 and X11.
The vendored CUI revision lacks that API. Revisit it after the Notes workflow
and desktop publishing work.

[apps]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/apps.md
[recipes]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/recipes.md
[getting-started]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/getting-started.md
[theming]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/theming.md
[file-dialogs]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/getting-started.md#7-files-and-links
[shortcuts]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/apps.md#your-own-shortcuts-and-input
[shipping]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/apps.md#shipping
[desktop-id]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/getting-started.md#your-apps-name-and-icon-on-linux
[window-api]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/apps.md#the-window
[app-options]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/apps.md#two-ways-to-run
[limitations]: https://github.com/yappologistic/CUI/blob/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441/docs/faq.md#what-can-it-not-do
