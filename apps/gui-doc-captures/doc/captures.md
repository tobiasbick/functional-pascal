# Gallery captures for the GUI handbook

## Capture session

Run from the repository root with the [local FPAS SDK and native GUI prerequisites](../../../docs/build/development.md).
The [project](../GuiDocCaptures.fpasproj) builds the capture sources and the
gallery units. Its entry point opens a CUI software-rendered window at scale
one with reduced motion. On Linux, use a desktop display or Xvfb.

```sh
dotnet run --project apps/gui-doc-captures/GuiDocCaptures.fpasproj -- .temp-data/ui-selection selection
```

The output directory receives one full-frame PNG and one UTF-8 `.crop` file
per widget view. Each crop records four integer coordinates in pixels, in
left, top, right, bottom order. The right and bottom edges are exclusive.
Use a fresh output directory to keep the captures from different runs separate.

| Implementation | Responsibility |
| --- | --- |
| [main.fpas](../src/main.fpas) | Starts the gallery, injects native input and dispatches to the chapter recipe. |
| [Chapters.fpas](../src/Chapters.fpas) | Parses arguments, resolves chapter names and selects viewport heights. |
| [Gestures.fpas](../src/Rendering/Gestures.fpas) | Provides the live session's keyboard and pointer gestures to interactive recipes. |
| [Presented.fpas](../src/Rendering/Presented.fpas) | Steps frames, posts messages and locates visible controls. |
| [Regions.fpas](../src/Rendering/Regions.fpas) | Captures frames and calculates crop bounds from presented controls. |

## Chapter recipes

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

Actions opens menus and the speed dial, arms the confirmation button and
reveals the swipe actions. The capture steps live in the following units.

<details>
<summary>Chapter sources</summary>

- [Actions.fpas](../src/Pages/Actions.fpas)
- [AiAndMotion.fpas](../src/Pages/AiAndMotion.fpas)
- [Charts.fpas](../src/Pages/Charts.fpas)
- [CreativeAndCanvas.fpas](../src/Pages/CreativeAndCanvas.fpas)
- [DeveloperTools.fpas](../src/Pages/DeveloperTools.fpas)
- [FeedbackAndDataDisplay.fpas](../src/Pages/FeedbackAndDataDisplay.fpas)
- [Foundations.fpas](../src/Pages/Foundations.fpas)
- [InputsAndPickers.fpas](../src/Pages/InputsAndPickers.fpas)
- [InstrumentsAndCapture.fpas](../src/Pages/InstrumentsAndCapture.fpas)
- [OverlaysAndDisclosure.fpas](../src/Pages/OverlaysAndDisclosure.fpas)
- [Overview.fpas](../src/Pages/Overview.fpas)
- [SelectionAndNavigation.fpas](../src/Pages/SelectionAndNavigation.fpas)
- [SmartInputs.fpas](../src/Pages/SmartInputs.fpas)
- [SocialAndVoice.fpas](../src/Pages/SocialAndVoice.fpas)
- [SourceTools.fpas](../src/Pages/SourceTools.fpas)
- [Workspaces.fpas](../src/Pages/Workspaces.fpas)

</details>

## Crop and check the images

Install Pillow for the Python interpreter used by the [crop command](../src/crop.py).
Run the command on a completed capture directory:

```sh
python apps/gui-doc-captures/src/crop.py .temp-data/ui-selection docs/ui/gui/selection/images
```

The cropper writes RGB PNGs into the chapter's `images` directory. It checks
that each rectangle fits within its frame and rejects single-color crops.
Review the results at their original size. If a frame is blank, rerun the
capture with the gallery window visible.

Some widgets are documented in another chapter. Place these images in the
corresponding handbook directory after cropping:

| Captured in | Image | Document in |
| --- | --- | --- |
| Audio | `guided-tour.png` | Overlays |
| AI | `composer-ai.png` | Voice |
| Smart inputs | `text-field-affixes.png` | Inputs |
| App chrome | `toolbar-editor.png` | Canvas and bars |
| App chrome | `formatted-text.png` | Text and display |
| Overview | `sidebar-appearance.png`, `sidebar-appearance-dark.png` | Navigation |

The [GUI documentation gap](../../../docs/future/gui-app-gaps.md#widget-documentation-captures)
tracks the portable FPAS crop operation.

## Verification

The [application tests](../tests/test_capture.py) check bounds, blank frames,
pixel preservation and the default Actions capture through both commands.
Consecutive full-frame PNGs from the same presented frame must have identical pixels.
Build the Debug project first. On headless Linux run the suite under Xvfb to
include the native gallery test.

```sh
dotnet build apps/gui-doc-captures/GuiDocCaptures.fpasproj
python apps/gui-doc-captures/tests/test_capture.py
```

## Check the widget examples

The default [documentation checker run](../../../docs/pascal/tools/documentation-examples.md#invocation-and-bootstrap)
compiles the examples under `docs/ui` directly from their Markdown fences. It
uses the UI and managed GUI assemblies copied when building
[doc-examples](../../doc-examples/).
