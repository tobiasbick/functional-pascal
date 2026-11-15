# Data grid

Use the filter bar to collect search and filter criteria. Use a data grid for
rows realized on demand, and a tree table for nested rows with additional
columns. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Filter bar](filter-bar.md) | Collect selected values by field and optional search text. |
| [Data grid](data-grid.md) | Show many rows with sort, selection, resizing and editable cells. |
| [Tree table](tree-table.md) | Show nested rows with disclosure, selection and optional lazy children. |
| [Tree cell](tree-cell.md) | Indent and expand a name in a table whose rows the model builds. |

The application owns the displayed order and selection. When it changes the
order, translate position-based proposals through the order that produced them.

## See also

- [UI handbook](../README.md)
- [Data display](../data-display/README.md)
- [Data grid in the gallery](../../../examples/ui/gui/gallery/Pages/DataGrid/View.fpas)
