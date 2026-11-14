# Desktop GUI application gaps

The first target is a small FPAS Notes app that opens and saves files, handles
keyboard shortcuts and asks about unsaved changes before closing. Use the
[GUI handbook](../pascal/libraries/gui.md) for the supported API and the
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
  [table API](../../src/Fpas.Ui/README.md#data-display) and show when to use a
  [data-grid row factory](../../src/Fpas.Ui/Widgets/Collections/DataGrids.fpas)
  for visible rows in large lists.
- [ ] Restore the saved appearance, accent and split ratio before the first
  frame. Show how to read and write application settings with .NET I/O.
  Keep defaults for missing or invalid settings and handle write failures
  through an application message. Dock layouts can use the existing
  [Dockspace layout callback](../../src/Fpas.Ui/Widgets/Desktop/Dockspaces.fpas).
- [ ] Add short, runnable examples for background progress and time-driven
  updates. Use [StartBackground and typed Post](../../src/Fpas.Ui.Mvu/README.md#background-effects)
  for progress, errors and timer ticks. Demonstrate cancellation and updates
  without user input. The
  [gallery countdown](../../examples/ui/gui/gallery/Pages/Disclosure/Clock.fpas)
  already uses this pattern. Add a direct frame scheduling API when a use case
  needs a repaint without a model change.

Use the Notes app to verify opening, editing, saving, reopening and restoring
settings. Cover button and keyboard submission, selection after sorting, and
both ways to handle unsaved changes. Keep the examples small enough to build
through the documented SDK workflow.

## Document commands and window close

- [ ] Route a native close request through the application's update before
  shutting down. The [GUI host](../../src/Fpas.Ui.Gui/Internal/Hosting/Session.fpas)
  currently starts cleanup immediately. Let the application approve the
  request, cancel it or show a confirmation and save first. Verify the title-bar
  close action, clean and dirty documents, repeated close requests, failed or
  cancelled saves, and normal cancellation and joining of background work once
  the close is approved.
- [ ] Add typed Save File and Choose Folder commands beside the existing
  [Open File command](../../src/Fpas.Ui.Gui/FileDialogs.fpas). Use CUI's native
  chooser, including its default filename for Save. Exercise confirmation,
  cancellation, Unicode paths and filters on Windows and Linux. Add a real
  Save As flow to Notes. [CUI file dialogs][file-dialogs]
- [ ] Let applications bind keyboard combinations such as primary modifier
  plus S to typed messages. The command palette has its own binding and
  recording a shortcut produces a value. Define how app bindings yield to text
  editors, active overlays and widget shortcuts. Verify that a shortcut fires
  once per activation and that disabled commands and covered controls stay
  inactive. [CUI keyboard shortcuts][shortcuts]

## Desktop publishing and window configuration

- [ ] Provide and document a supported Windows GUI executable without a console
  window. The [FPAS compile task](../../src/Fpas.Build.Tasks/FpasCompile.cs)
  currently accepts `Exe` and `Library`. Verify the executable launched
  directly from Explorer for the supported desktop publish modes. Cover the
  project's icon and version metadata in the publishing instructions.
  [CUI shipping guide][shipping]
- [ ] Expose Linux application identity through GUI options, the managed
  window and the bridge to `cui_app_desc.app_id`. Match the installed
  `.desktop` filename and icon. Verify the resulting X11 `WM_CLASS` and
  desktop grouping. CUI reads the ID at window creation and ignores it on
  Windows. [CUI desktop identity][desktop-id]
- [ ] Expose runtime title and logical size changes, minimum dimensions and
  fixed-size windows through the GUI host. Verify that an app can restore a
  saved size and update the title when it opens another file. Leave native
  handle and backend access for a concrete platform integration such as a tray
  icon. [CUI window API][window-api]

## Appearance and accessibility

- [ ] Make the GUI's System choice follow the OS appearance on Windows and
  apply changes to the model-selected palette. Verify startup, a live OS
  change and switching back to an explicit Light or Dark choice. Document
  the fallback for X11, where the pinned CUI backend only has a limited
  startup hint. [CUI app options][app-options]
- [ ] Expose reduced motion through the public GUI API. The native binding
  already has `SetReducedMotion`. Verify it with a running animation. Expose
  font-role overrides and fallback fonts where an application needs them.
  Test visible CJK glyphs with a supplied fallback. Font coverage and
  bidirectional editing are separate problems. [CUI typography and motion][theming]
- [ ] Establish a screen-reader plan before treating the GUI as accessible for
  public apps. CUI has no accessibility tree. FPAS snapshots expose bounds,
  roles and values for inspection. A screen reader also needs platform
  semantics, focus events and actions. Start by validating a form, a modal and
  a list with a real screen reader. Track IME and complex-script editing
  separately. [CUI limitations][limitations] ·
  [FPAS platform limits](../pascal/libraries/gui.md#publishing-and-platform-limits)

## Deferred window chrome

The [optional title bar](cui-titlebar.md) has a separate plan. The local CUI
prototype at `00eddec` adds an experimental title bar on Win32 and X11.
The vendored CUI revision lacks that API. Revisit it after the Notes workflow
and desktop publishing work.

[apps]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/apps.md
[recipes]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/recipes.md
[getting-started]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/getting-started.md
[theming]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/theming.md
[file-dialogs]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/getting-started.md#7-files-and-links
[shortcuts]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/apps.md#your-own-shortcuts-and-input
[shipping]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/apps.md#shipping
[desktop-id]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/getting-started.md#your-apps-name-and-icon-on-linux
[window-api]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/apps.md#the-window
[app-options]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/apps.md#two-ways-to-run
[limitations]: https://github.com/yappologistic/CUI/blob/a8f71c05be29eb441bf3ff72dba67b279e68384f/docs/faq.md#what-can-it-not-do
