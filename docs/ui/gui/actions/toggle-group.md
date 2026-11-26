# Toggle group

Group independent flags such as text formatting or exclusive choices such as alignment.

![Toggle group in the FPAS gallery](images/toggle-group.png)

## Example

```pascal
unit Docs.Ui.Actions.ToggleGroup;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Actions.ToggleGroups as T;

// Builds the example view.
// ## Parameters
// - `Selected`: Current application value.
// ## Returns
// The widget with its message type.
public function View(Selected: array of bool): E.Element<array of bool>;
begin
  return T.ToggleGroup<array of bool>('format', [new T.Item('Bold'), new T.Item('Italic')],
    Selected, function(Flags: array of bool): array of bool => Flags);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Exclusive / IconOnly` | false / false |
| `ControlSize / Enabled` | Medium / true |
| `Items` | 1 to 32 positional items |

## Interaction

Pass one selected flag per item. Exclusive mode requires exactly one
selected item. Tab visits enabled items and the arrow keys move focus
horizontally. Click, Enter or Space proposes a fresh array of flags. Keep
the array in the model after accepting a proposal.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/ToggleGroups.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Actions/Widgets/ToggleGroup.fpas)
- [Actions](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Actions/`
  - [ToggleGroup.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Actions/ToggleGroup.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Actions/`
  - [NativeMethods.ToggleGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Actions/NativeMethods.ToggleGroup.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Actions/`
  - [CuiWindow.ToggleGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.ToggleGroup.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [toggle_group.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/toggle_group.c)
  - [toggle_group.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/toggle_group.h)
- `tests/Fixtures/gui-host/`
  - [ToggleGroupContract.fpas](../../../../tests/Fixtures/gui-host/ToggleGroupContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Actions/`
  - [CuiToggleGroupTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Actions/CuiToggleGroupTests.cs)
