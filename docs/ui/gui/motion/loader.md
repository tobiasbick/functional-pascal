# Loader

The gallery shows dots, bars, pulse, orbit and grid from left to right.
Available in **GUI**.

![Five native loader styles in the FPAS gallery](images/loader.png)

## Example

```pascal
unit Docs.Ui.Motion.Loader;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Feedback.Loaders as W;

// Shows a loader for pending work.
// ## Parameters
// - `Style`: One of Dots, Bars, Pulse, Orbit or Grid.
// ## Returns
// The chosen native loader.
public function View(Style: W.Animation): E.Element<string>;
begin
  return W.Loader<string>('wait', Style, new W.Options(Size := 28.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Style` | Choose one of the five animation values. |
| `Size / Color` | 0 / Automatic. Zero uses the native size. |

## Interaction

Include the loader while work is pending. The application tracks the
work, and CUI animates the indicator.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/Loaders.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Motion/View.fpas)
- [Shimmer text](shimmer-text.md)
- [Spinner](../feedback/spinner.md)
- [Motion](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Feedback/`
  - [Loader.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/Loader.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.Loader.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.Loader.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.Loader.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Loader.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [loader.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/loader.c)
  - [loader.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/loader.h)
