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
| Buttons, menus, theme controls and related actions | [Actions](actions/README.md) |
| Checkboxes, switches, radio controls, sliders, chips and ratings | [Selection](selection/README.md) |
| Sidebar, tabs, breadcrumb, pagination and stepper | [Navigation](navigation/README.md) |
| Text editors, credentials, form fields and single-date calendars | [Inputs](inputs/README.md) |
| Multi-select, date ranges, time controls and measurement sliders | [Pickers](pickers/README.md) |
| Dialogs, sheets, popovers, command menus and notifications | [Overlays](overlays/README.md) |
| Inline previews, expanding content, onboarding, undo and reveal controls | [Disclosure](disclosure/README.md) |
| Alerts, progress indicators, loading placeholders and status labels | [Feedback](feedback/README.md) |
| Tables, trees, lists and accordion sections | [Data display](data-display/README.md) |
| Text display, data grids, charts and other areas | [Widget inventory by area](../cui-widget-inventory.md#widget-inventory) |
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
