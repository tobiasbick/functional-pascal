# Counter

Use a counter for a value that changes over time. The gallery adds a sale
to a revenue total with grouped thousands.
Available in **GUI**.

![Revenue counter with Add a sale in the FPAS gallery](images/counter.png)

## Example

```pascal
unit Docs.Ui.SocialAndMedia.Counter;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Charts as C;
uses Fpas.Ui.Widgets.Feedback.Counters as W;

// Shows current revenue.
// ## Parameters
// - `Revenue`: Current total from the model.
// ## Returns
// The native counter.
public function View(Revenue: double): E.Element of string;
begin
  return W.Counter of string('revenue', Revenue,
    new W.Options(Format := new C.ValueFormat(Prefix := '$'),
      Size := 34.0F, GroupThousands := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Format / Color` | null / Automatic. A value format controls precision and literal prefix or suffix. |
| `Font` | `Graphics.Font.Display`. CUI also uses Display for Body. |
| `Size / GroupThousands` | 0 / false. Zero uses 32 logical units. GroupThousands separates thousands with commas. |

## Interaction

Pass a new value with the same ID when the total changes. CUI animates
the display to the new value. The button that changes the total belongs
to the application.

The format can use a Unicode currency symbol or unit.
[CUI-072](../../../future/cui-upstream-bugs.md#cui-072)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_counter)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/Counters.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Social/View.fpas)
- [Countdown](../disclosure/countdown.md)
- [Social and media](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Feedback/`
  - [Counter.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/Counter.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.Counter.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.Counter.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.Counter.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Counter.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [counter.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/counter.c)
  - [counter.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/counter.h)
