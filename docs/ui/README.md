# FPAS user interface handbook

Build desktop applications with FPAS and CUI. Widgets render a model and send
typed messages. Your update function accepts those messages, changes the model
and returns any commands. The terminal host uses the same model and message
pattern for its supported widgets.

## First window

After the [SDK bootstrap](../build/development.md#build-an-fpas-application), run
the [counter example](../../examples/ui/gui/first-app/FirstGuiApp.fpasproj) from
the repository root.

```sh
dotnet run --project examples/ui/gui/first-app/FirstGuiApp.fpasproj
```

In the [source](../../examples/ui/gui/first-app/main.fpas), `Gui.Run` starts
with a model of zero. `View` shows the count and an Add one button. Activating
that button sends `Msg.Increment`. `Update` returns the incremented count, and
the next view shows it. Close sends a command that ends the application.

## Widgets

| Area | Reference |
| --- | --- |
| Metric tiles, the workspace form and team card | [Overview](overview/README.md) |
| Buttons, menus, theme controls and related actions | [Actions](actions/README.md) |
| Checkboxes, switches, radio controls, sliders, chips and ratings | [Selection](selection/README.md) |
| Sidebar, tabs, breadcrumb, pagination and stepper | [Navigation](navigation/README.md) |
| Table of contents and scroll anchors | [Reader](reader/README.md) |
| Menu bar, toolbar, document tabs, status bar, docking and split panes | [App chrome](app-chrome/README.md) |
| Text roles, formatted labels, icons, emoji, keycaps, images and drawing | [Text and display](text-and-display/README.md) |
| Layered stacks, dividers, reading direction, spacing and scroll views | [Layout](layout/README.md) |
| Text editors, credentials, form fields and single-date calendars | [Inputs](inputs/README.md) |
| Inline edits, scrub and money fields, shortcut recording and emoji | [Smart inputs](smart-inputs/README.md) |
| Multi-select, date ranges, time controls and measurement sliders | [Pickers](pickers/README.md) |
| Dialogs, sheets, popovers, guided tours and notifications | [Overlays](overlays/README.md) |
| Inline previews, expanding content, onboarding, undo and reveal controls | [Disclosure](disclosure/README.md) |
| Alerts, progress indicators, loading placeholders and status labels | [Feedback](feedback/README.md) |
| Tables, trees, lists and accordion sections | [Data display](data-display/README.md) |
| Filter bars, virtual data grids, tree tables and tree cells | [Data grid](data-grid/README.md) |
| Line and bar charts, donuts, gauges and sparklines | [Charts](charts/README.md) |
| Heatmaps, flow diagrams, rankings and radial gauges | [More charts](more-charts/README.md) |
| Readouts, dot-matrix charts and displays, and event tracks | [Instruments](instruments/README.md) |
| Rotary controls, mixer faders and audio level meters | [Audio](audio/README.md) |
| File drops, profile photos and signatures | [Capture](capture/README.md) |
| File browsers, path bars, file pickers and terminal output | [Files](files/README.md) |
| Kanban boards, calendars and Gantt charts | [Planning](planning/README.md) |
| Source display, Markdown, code editing, diffs, hex and logs | [Code](code/README.md) |
| Property inspection, profiling, curves, gradients and timelines | [Editing tools](editing-tools/README.md) |
| Traces, dependencies, diagnostics, filters and schedules | [Developer tools](developer-tools/README.md) |
| Test results, coverage, HTTP controls and service health | [More developer tools](more-developer-tools/README.md) |
| Ratio and color sliders, palette, output options and seed | [Creative controls](creative-controls/README.md) |
| Node editing, collaborator indicators and action bars | [Canvas and bars](canvas-and-bars/README.md) |
| Conversation, presence, scrolling figures and media views | [Social and media](social-and-media/README.md) |
| Composer, voice messages, waveforms and call controls | [Voice](voice/README.md) |
| Reasoning, agent state, sources and response controls | [AI](ai/README.md) |
| Text effects, loaders and compact audio and navigation controls | [Motion](motion/README.md) |
| CUI coverage and acceptance by area | [Widget inventory](../cui-widget-inventory.md#widget-inventory) |
| Shared layout, styles, themes and API contracts | [FPAS widget reference](../../src/Fpas.Ui/README.md) |

The widget pages show the FPAS gallery at scale 1 with the software renderer.
The FPAS examples return widgets for a view like the counter's `View`
function, or toast commands for an update. Each page links to the public API
and the complete gallery example.
The [capture workflow](../../apps/gui-doc-captures/README.md) regenerates its
screenshots.

## Application guides

- [Desktop GUI](../pascal/libraries/gui.md) covers project setup, the event loop,
  background work, snapshots and publishing.
- [GUI widget gallery](../../examples/ui/gui/gallery/README.md) shows the full
  collection in one application.
- [Terminal widget gallery](../../examples/ui/tui/gallery/README.md) shows the
  supported terminal subset.
