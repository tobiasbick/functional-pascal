# Progress bar

Show how much of a task is complete. The terminal uses a cell track with an optional label and rounded percentage.

![Uploading assets and exporting a clip with separate labeled progress tracks](../../../../examples/ui/tui/gallery/panels.png)

## Example

```pascal
unit Docs.Tui.Widgets.ProgressBar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Feedback.ProgressBars as W, Fpas.Ui.Styles as S;

// Builds the progress bar example.
// ## Parameters
// - `Fraction`: Completed fraction, from zero to one.
// ## Returns
// The resulting view.
public function View(Fraction: float): E.Element<string>;
requires Fraction >= 0.0F and Fraction <= 1.0F;
begin
  return W.ProgressBar<string>('upload', Fraction,
    new W.Options(Label := 'Uploading assets', ShowValue := true, Tone := S.Tone.Info));
end function;

end unit;
```

## Interaction

Pass a fraction from zero to one. Update it when work reports progress. The bar has no focus stop. In the gallery, the page binds Left and Right to adjust the sample. The TUI supports determinate progress with a fixed or available width.

## See also

- [Feedback](README.md)
- [Shared API](../../widgets.md#alerts-progress-and-spinners)
- [GUI progress bar](../../gui/feedback/progress-bar.md)
- [Gallery source](../../../../examples/ui/tui/gallery/main.fpas)
