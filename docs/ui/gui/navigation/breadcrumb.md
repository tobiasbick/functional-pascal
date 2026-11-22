# Breadcrumb

Use a breadcrumb to show the current location and let the user go back to an
ancestor. Available in **GUI** and [TUI](../../tui/navigation/breadcrumb.md).

![Breadcrumb in the FPAS gallery](images/breadcrumb.png)

## Example

```pascal
unit Docs.Ui.Navigation.Breadcrumb;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Navigation.Breadcrumbs as C;

// Builds the breadcrumb example.
// ## Returns
// The widget for the application view.
public function View(): E.Element<int>;
begin
  return C.Breadcrumb<int>('breadcrumbs', ['Home', 'Projects', 'Settings'],
    function(Index: int): int => Index);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | A nonempty array in path order. The last caption is the current location. |
| `OnNavigate` | Receives the zero-based position of the activated ancestor. |
| `Enabled` | true. |

## Interaction

Click an ancestor or focus it with Tab and press Enter or Space. Position identifies the link, so captions may repeat. Use `ItemId(Id, Index)` to focus one. The final caption is informational.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/Breadcrumbs.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Navigation/Widgets/Breadcrumb.fpas)
- [Navigation](README.md)

## Source files

- [Shared gallery project trail](../../../../examples/ui/gui/gallery/Shared/BreadcrumbExample.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Navigation/`
  - [Breadcrumb.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Navigation/Breadcrumb.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Navigation/`
  - [NativeMethods.Breadcrumb.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Navigation/NativeMethods.Breadcrumb.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Navigation/`
  - [CuiWindow.Breadcrumb.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Navigation/CuiWindow.Breadcrumb.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/navigation/`
  - [breadcrumb.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/breadcrumb.c)
  - [breadcrumb.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/breadcrumb.h)
