# Spinner

Use a spinner to indicate work in progress when the total is unavailable. The gallery shows three sizes and several animation styles. See the [terminal spinner](../../tui/feedback/spinner.md).

![Spinner in the FPAS gallery](images/spinner.png)

![Dither spinner styles and inline examples in the FPAS gallery](images/spinner-styles.png)

## Example

```pascal
unit Docs.Ui.Feedback.Spinner;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Feedback.Spinners as W;

// Builds the spinner example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.Spinner of string('syncing', 22.0F, new W.Options(Animation := W.Style.Orbit));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Size` | 0 uses the native default diameter of 20 logical points. Supply a finite nonnegative size. |
| `Animation` | Arc. The gallery also shows Comet, DotMatrix, Orbit and other styles. |
| `Color / Track` | Automatic / Automatic. |
| `Inverse` | false. Enable it for the inverse palette used on dark surfaces. |

## Interaction

The spinner draws a native activity animation and has no input. Choose when to include or remove it in the view from the application model.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_spinner)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/Spinners.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/Loading.fpas)
- [Status spinner](status-spinner.md)
- [Feedback](README.md)

## Source files

- [Spinner.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/Spinner.fpas)
