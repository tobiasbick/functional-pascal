# Stepper

Use a stepper to show progress through an ordered workflow. Available in **GUI**.

![Stepper in the FPAS gallery](images/stepper.png)

## Example

```pascal
unit Docs.Ui.Navigation.Stepper;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Navigation.Steppers as C;

// Builds the stepper example.
// ## Parameters
// - `Current`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Current: int): E.Element of string;
begin
  return C.Stepper of string('workflow-steps', ['Account', 'Workspace', 'Invite team', 'Done'], Current);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Steps` | A nonempty array of step captions. |
| `Current` | Zero-based step in progress. Pass the step count once all steps are complete. |

## Interaction

The stepper marks preceding steps complete. In a narrow view it keeps the current caption visible and shows markers for the other steps. The indicator is read-only. Handle navigation in separate controls such as the gallery's Back and Next buttons.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_stepper)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/Steppers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Navigation/Widgets/Stepper.fpas)
- [Navigation](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Navigation/`
  - [Stepper.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Navigation/Stepper.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Navigation/`
  - [NativeMethods.Stepper.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Navigation/NativeMethods.Stepper.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Navigation/`
  - [CuiWindow.Stepper.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Navigation/CuiWindow.Stepper.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/navigation/`
  - [stepper.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/stepper.c)
  - [stepper.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/stepper.h)
