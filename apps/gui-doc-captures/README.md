# Capture widgets for the UI handbook

The capture project opens the FPAS gallery with the CUI software renderer at
scale 1 with reduced motion. The Actions capture also opens the menus and
speed dial, arms the confirmation button and reveals the swipe actions.
For each screenshot, it saves a PNG and the crop bounds from the presented
control IDs.

Use the [SDK bootstrap and GUI prerequisites](../../docs/build/development.md).
Run the following commands from the repository root on a desktop with a display.
The crop step requires Python with Pillow installed.

```sh
dotnet run --project apps/gui-doc-captures/GuiDocCaptures.fpasproj -- .temp-data/ui-selection selection
python apps/gui-doc-captures/crop.py .temp-data/ui-selection docs/ui/selection/images
```

The chapter argument accepts `actions` (the default), `selection`, `navigation`,
`inputs`, `pickers`, `overlays`, `disclosure`, `feedback`, `data-display`,
`charts` or `more-charts`. More charts scrolls the gallery to capture the
lower examples.
Use a fresh capture directory for each run and crop into the chapter's
`images` folder. Review the output at its original size.
The crop keeps the widget and its open menu entries. If a capture is blank
while the desktop is busy, rerun the capture with the gallery window visible.

The crop step uses Pillow because the shared .NET runtime and FPAS GUI
currently lack a portable bitmap crop operation. See the
[GUI documentation gap](../../docs/future/gui-app-gaps.md#widget-documentation-captures).

## Check the widget examples

The default [documentation checker run](../../docs/pascal/tools/documentation-examples.md#invocation-and-bootstrap)
compiles the examples under `docs/ui` directly from their Markdown fences. It
uses the UI and managed GUI assemblies copied when building
[doc-examples](../doc-examples/).
