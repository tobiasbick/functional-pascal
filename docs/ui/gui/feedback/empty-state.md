# Empty state

Use an empty state to explain an empty area and offer the next action when one is available. Available in **GUI**.

![Empty state in the FPAS gallery](images/empty-state.png)

## Example

```pascal
unit Docs.Ui.Feedback.EmptyState;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Containers.EmptyStates as W;

// Builds the empty state example.
// ## Returns
// The widget for the application view.
public function View(): E.Element<string>;
begin
  return W.EmptyState<string>('no-projects', 'No projects yet',
    new W.Options(Description := 'Create a project to group screens and builds.', ActionLabel := 'New project'),
    function(): string => 'create-project');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title / Description` | A heading and optional description. Description defaults to empty. |
| `Icon` | Empty. Supply a CUI glyph for the illustration position. |
| `ActionLabel / OnAction` | Empty / null. Supply both to add an action. |
| `Enabled` | true. |

## Interaction

The optional button sends the supplied message. Apply it in the update and replace the empty state when the collection has content. Use `ActionId(Id)` to address that button for focus.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/EmptyStates.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/View.fpas)
- [Feedback](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Containers/`
  - [EmptyState.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Containers/EmptyState.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Containers/`
  - [NativeMethods.EmptyState.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Containers/NativeMethods.EmptyState.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Containers/`
  - [CuiWindow.EmptyState.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Containers/CuiWindow.EmptyState.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/containers/`
  - [empty_state.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/empty_state.c)
  - [empty_state.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/empty_state.h)
