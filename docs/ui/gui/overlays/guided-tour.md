# Guided tour

Use a guided tour to highlight controls and explain them in order. The gallery
walks through the Cutoff knob, master meter and Bass fader. Available in **GUI**.

![First step of the Audio tour in the FPAS gallery](images/guided-tour.png)

## Example

```pascal
unit Docs.Ui.Overlays.GuidedTour;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Overlays.Tours as W;

// Adds a tour to an audio panel.
// ## Parameters
// - `Panel`: Base view containing the cutoff and master controls.
// - `Current`: Zero-based step, or minus one when the tour is closed.
// ## Returns
// The panel with native navigation proposals as application messages.
public function View(Panel: E.Element of W.Change; Current: int): E.Element of W.Change;
requires Current >= -1 and Current < 2;
begin
  return W.Tour of W.Change('audio-tour', Panel,
    [new W.Step(Target := 'cutoff', Title := 'Turn a knob', Body := 'Drag up and down.'),
     new W.Step(Target := 'master', Title := 'Watch the levels')],
    Current, function(Change: W.Change): W.Change => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Steps / Current` | Supply at least one step. Use minus one when closed. |
| `Target / Position` | Empty / BottomStart. Set the stable ID of a control in the base view. An empty or absent target centers the card. CUI fits the card within the window. |
| `Title / Body` | Empty. Supply the heading and supporting text for each step. |
| `Width / Padding` | 0 / 0. CUI uses 300 logical points for the card and 6 around the target. |
| `BlockTarget` | false. Set true to block pointer input in the spotlight too. |
| `NextLabel / BackLabel / DoneLabel` | null displays Next, Back and Done. Pass literal captions, including an empty string for a blank caption. |
| `CountFormat` | null displays "step of total". Use up to two `%d` slots (step, total), plus literal `%%`. An empty string hides the count text. |

## Interaction

Set the current step to zero to start. CUI draws the spotlight and handles
Back, Next, Done and dismissal. Store `Change.Step` to accept the proposal.
`Change.Action` distinguishes completion from skipping, which both set the
step to minus one.

The GUI host resolves the target from the controls in the current frame.
For native callers, the matching CUI identity comes from `cui_last_id` after
the target control. The knob and fader targets highlight their interactive
areas. The meter target highlights the entire meter.

Tab moves between the tour buttons. With tour focus, Right advances, Left
goes back, and Escape ends the tour. Enter or Space activates the focused
button. Click the spotlighted control to use its normal input when
`BlockTarget` is false. While the control holds focus, its arrow keys stay
with the control. The dim area blocks pointer and wheel input.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_tour)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Overlays/Tours.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Audio/View.fpas)
- [Overlays](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Overlays/`
  - [Tour.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Overlays/Tour.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Overlays/`
  - [NativeMethods.Tour.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Overlays/NativeMethods.Tour.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Overlays/`
  - [CuiWindow.Tour.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Tour.Types.cs)
  - [CuiWindow.Tour.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Tour.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [tour.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/tour.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/overlays/`
  - [tour.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/tour.c)
  - [tour.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/tour.h)
- `tests/Fixtures/gui-host/`
  - [TourContract.fpas](../../../../tests/Fixtures/gui-host/TourContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Overlays/`
  - [CuiTourTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Overlays/CuiTourTests.cs)
