# Property inspector

Use a property inspector to edit typed values in groups. The gallery shows
a crate's transform, material and shadows. Available in **GUI**.

![Crate properties in the FPAS gallery](images/property-inspector.png)

## Example

```pascal
unit Docs.Ui.EditingTools.PropertyInspector;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Editors.PropertyInspectors as W;

// Builds the inspector for the selected object.
// ## Parameters
// - `Groups`: The object's properties and expansion state.
// ## Returns
// The inspector with edits and expansion changes as messages.
public function View(Groups: array of W.Group): E.Element of W.Event;
begin
  return W.PropertyInspector of W.Event('object', Groups,
    function(Change: W.Event): W.Event => Change, new W.Options(LabelWidth := 126.0F));
end function;

// Creates editable material properties.
// ## Returns
// A group with a bounded number and a checkbox.
public function Material(): W.Group;
begin
  return new W.Group('Material', [
    new W.Property('Roughness', W.Value.Number(0.65F),
      Minimum := 0.0F, Maximum := 1.0F, Step := 0.01F),
    new W.Property('Cast shadows', W.Value.Boolean(true), Checkbox := true)
  ]);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Group.Open / Collapsible` | Both true. |
| `Property.Current` | Supply Number, Integer, Text, Boolean, Choice, Color or Vector. Vectors have two to four components. |
| `Minimum / Maximum / Step / Digits` | 0 / 0 / 0 / minus one. Two zero bounds use the native unbounded range. Minus one derives precision from the step. |
| `Capacity / Enabled / Checkbox` | 256 UTF-8 bytes including the terminator / true / false. |
| `Width / LabelWidth` | Fill / 0. Zero uses the native label proportion. |

## Interaction

CUI chooses the editor from the value type. Store an `Expansion` proposal
on the corresponding group. Store the typed value from `Edit` at its group
and row indices, and render the resulting groups.

Replacing a text value or changing its capacity resets that row's selection
and undo history. Ordinary edits retain undo/redo; rejecting an edit restores
the model value. Hidden or removed text rows release their editing state.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Editors/PropertyInspectors.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/EditingTools/View.fpas)
- [Editing tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Editors/`
  - [PropertyInspector.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Editors/PropertyInspector.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Editors/`
  - [NativeMethods.PropertyInspector.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Editors/NativeMethods.PropertyInspector.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Editors/`
  - [CuiWindow.PropertyInspector.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiWindow.PropertyInspector.Types.cs)
  - [CuiWindow.PropertyInspector.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiWindow.PropertyInspector.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [property_text_replacement.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/property_text_replacement.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/editors/`
  - [property_inspector.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/editors/property_inspector.c)
  - [property_inspector.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/editors/property_inspector.h)
- `tests/Fpas.Ui.Gui.Cui.Tests/Inputs/`
  - [CuiPropertyInspectorTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiPropertyInspectorTests.cs)
