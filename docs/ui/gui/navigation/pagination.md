# Pagination

Use pagination to change pages within a collection. Available in **GUI** and
[TUI](../../tui/navigation/pagination.md).

![Pagination in the FPAS gallery](images/pagination.png)

## Example

```pascal
unit Docs.Ui.Navigation.Pagination;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Navigation.Paginations as C;

// Builds the pagination example.
// ## Parameters
// - `Page`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Page: int): E.Element<int>;
begin
  return C.Pagination<int>('pages', Page, 12, function(Index: int): int => Index);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Page` | Current zero-based index. |
| `PageCount` | A positive page count. Page must be below it. |
| `OnChange` | Receives the proposed zero-based page. |
| `Enabled` | true. |

## Interaction

Click a page button or focus it with Tab and press Enter or Space. CUI displays one-based page numbers and fits the buttons and ellipses to the available width. Previous and Next are disabled at the respective ends.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/Paginations.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Navigation/Widgets/Pagination.fpas)
- [Tabs](tabs.md)
- [Navigation](README.md)

## Source files

- [Shared twelve-page gallery example](../../../../examples/ui/gui/gallery/Shared/PaginationExample.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Navigation/`
  - [Pagination.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Navigation/Pagination.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Navigation/`
  - [NativeMethods.Pagination.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Navigation/NativeMethods.Pagination.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Navigation/`
  - [CuiWindow.Pagination.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Navigation/CuiWindow.Pagination.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/navigation/`
  - [pagination.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/pagination.c)
  - [pagination.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/pagination.h)
