# Format bar

The gallery edits color, emphasis, style, alignment and list formatting
and asks for confirmation before deletion.
Available in **GUI**.

![Text format bar in the FPAS gallery](images/format-bar.png)

## Example

```pascal
unit Docs.Ui.CanvasAndBars.FormatBar;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Canvas.FormatBars as W;

// Shows formatting for the current text selection.
// ## Parameters
// - `Current`: The format of the current selection.
// ## Returns
// The format bar with edits and actions as messages.
public function View(Current: W.Value): E.Element<W.Event>;
begin
  return W.FormatBar<W.Event>('format-selection', Current,
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Styles / Colors` | Both empty. Supply style parameters and tones to override the native menus. |
| `Footer / FooterKeys / DeleteQuestion` | All null. Supply strings to override the native hints and confirmation caption. |
| `HideDelete / HideMore / Enabled` | false / false / true. |

## Interaction

Apply `Event.Format` to the selection. Handle `ToggleLink` and `More`
in the application. `Delete` is true after the user confirms deletion.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Canvas/FormatBars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Canvas/View.fpas)
- [Canvas and bars](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Canvas/`
  - [FormatBar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Canvas/FormatBar.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Canvas/`
  - [NativeMethods.FormatBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Canvas/NativeMethods.FormatBar.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Canvas/`
  - [CuiWindow.FormatBar.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.FormatBar.Types.cs)
  - [CuiWindow.FormatBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.FormatBar.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/canvas/`
  - [format_bar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/format_bar.c)
  - [format_bar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/format_bar.h)
