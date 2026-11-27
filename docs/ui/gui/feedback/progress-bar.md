# Progress bar

Use a progress bar for a task with measurable progress or work that has no known total. Determinate progress is available in **GUI and TUI**.

![Progress bar in the FPAS gallery](images/progress-bar.png)

## Example

```pascal
unit Docs.Ui.Feedback.ProgressBar;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Feedback.ProgressBars as W;

// Builds the progress bar example.
// ## Parameters
// - `Fraction`: Current progress from zero through one.
// ## Returns
// The widget for the application view.
public function View(Fraction: float): E.Element of string;
begin
  return W.ProgressBar of string('upload', Fraction,
    new W.Options(Label := 'Uploading assets', ShowValue := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value` | A finite fraction from 0 through 1. |
| `Label / ShowValue` | Empty / false. ShowValue displays a rounded percentage for determinate progress. |
| `Indeterminate / Dither` | false / false. Indeterminate uses activity motion. Dither uses ordered dots for either mode. |
| `Tone` | Neutral. CUI uses the control-on color for a solid fill and the emphasis color for dither. |
| `Height` | 0 uses six logical units for a solid track and eight for dither. |
| `Width` | Content uses CUI's default sizing. Fill, Fixed, and Percent set the bar and its caption row width. |

## Interaction

Pass the fraction from the model and update it when the work reports progress.
Set `Indeterminate` when a total is unavailable. CUI uses a moving solid
segment or band of dots and omits the percentage. Reduced motion holds the
band of dots at the center. The solid segment keeps moving.

CUI interpolates the determinate fill. With dither, the dots drift until the
track is empty or full. Reduced motion holds the dots still. The GUI clips a
solid fill to its track when the track is narrower than its height.

Each bar is read-only. The TUI displays a one-row track with the optional label
and percentage above it. Fixed widths are clipped to the available cells.
Indeterminate mode, dither and custom track height require the GUI.

Use `Elements.Size` with width growth weights to share space among widgets in a
GUI row. [Sizing and units](../../layout/sizing.md#independent-gui-axes)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_progress)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/ProgressBars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/View.fpas)
- [Progress ring](progress-ring.md)
- [Meter](meter.md)
- [Feedback](README.md)
- [Terminal example](../../tui/feedback/progress-bar.md)

## Source files

- [Shared gallery example](../../../../examples/ui/gui/gallery/Shared/ProgressExample.fpas)

- [ProgressBar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/ProgressBar.fpas)
