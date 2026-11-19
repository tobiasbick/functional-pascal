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
python apps/gui-doc-captures/crop.py .temp-data/ui-selection docs/ui/gui/selection/images
```

The chapter argument accepts `overview`, `actions` (the default), `selection`, `navigation`,
`inputs`, `smart-inputs`, `pickers`, `overlays`, `disclosure`, `feedback`, `data-display`,
`charts`, `more-charts`, `instruments`, `audio`, `capture`, `data-grid`,
`files`, `planning`, `code`, `editing-tools`, `developer-tools`,
`more-developer-tools`, `creative-controls`, `canvas-and-bars`,
`social-and-media`, `voice`, `ai`, `motion`, `text-and-display`, `reader`,
`app-chrome` or `layout`. The longer pages scroll to
capture their lower examples. Code also captures the split and inline diff
layouts. Canvas and bars uses a fixed collaborator animation
time and includes the active bulk selection.
Audio captures the open guided tour for the Overlays chapter. Capture draws
a sample stroke in the signature pad.
Voice advances the simulated call and records a synthetic take to capture
the composer and an outgoing voice note.
AI opens the reasoning and tool details and the standalone model menu. It
also shows the agent status during the simulated reply. Motion captures both
values of the rolling-number example with reduced motion.
Text and display clicks the blue swatch. Reader jumps to Install. App
chrome opens the View menu after capturing the editor and workspace, then
scrolls to the window actions.
Smart inputs captures the message field before and after opening the emoji palette.
Overview captures the workspace and metrics as well as the sidebar in both
appearances. Layout captures the divider in the Notifications sheet.
Files creates a separate sample directory within the capture output and
lists those files.
Use a fresh capture directory for each run and crop into the chapter's
`images` folder. Move Audio's `guided-tour.png` to `docs/ui/gui/overlays/images`,
AI's `composer-ai.png` to `docs/ui/gui/voice/images` and Smart inputs'
`text-field-affixes.png` to `docs/ui/gui/inputs/images`. Move App chrome's
`toolbar-editor.png` to `docs/ui/gui/canvas-and-bars/images` and
`formatted-text.png` to `docs/ui/gui/text-and-display/images`. Move Overview's
two `sidebar-appearance` captures to `docs/ui/gui/navigation/images`.
Review the output at its original size.
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

## Source files

<details>
<summary>Browse 19 source files</summary>

- `apps/gui-doc-captures/`
  - [AiAndMotion.fpas](AiAndMotion.fpas)
  - [Chapters.fpas](Chapters.fpas)
  - [Charts.fpas](Charts.fpas)
  - [CreativeAndCanvas.fpas](CreativeAndCanvas.fpas)
  - [DeveloperTools.fpas](DeveloperTools.fpas)
  - [FeedbackAndDataDisplay.fpas](FeedbackAndDataDisplay.fpas)
  - [Foundations.fpas](Foundations.fpas)
  - [GuiDocCaptures.fpasproj](GuiDocCaptures.fpasproj)
  - [InstrumentsAndCapture.fpas](InstrumentsAndCapture.fpas)
  - [Overview.fpas](Overview.fpas)
  - [Presented.fpas](Presented.fpas)
  - [Regions.fpas](Regions.fpas)
  - [SelectionAndNavigation.fpas](SelectionAndNavigation.fpas)
  - [SmartInputs.fpas](SmartInputs.fpas)
  - [SocialAndVoice.fpas](SocialAndVoice.fpas)
  - [SourceTools.fpas](SourceTools.fpas)
  - [Workspaces.fpas](Workspaces.fpas)
  - [crop.py](crop.py)
  - [main.fpas](main.fpas)

</details>
