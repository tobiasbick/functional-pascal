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

## Data grid gallery page

The filter bar shows three fields and a search box. Its state is independent
of the 2,000-person grid. The grid uses a 420-unit viewport and creates the
visible rows on demand. Each name cell combines an avatar with an editor.
Each score cell combines a progress bar with a tabular label.

Equal sort keys keep their prior display order. FPAS keeps selections,
cursor and range origin by person identity. A name edit updates the order
immediately when the grid sorts by name.

The source tree composes expandable names and right-aligned totals in an
ordinary table. The following virtual tree example demonstrates lazy children,
keyboard navigation and the context menu.

## See also

- [UI handbook](../README.md)
- [Data display](../data-display/README.md)
- [Data grid in the gallery](../../../../examples/ui/gui/gallery/Pages/DataGrid/View.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#data-grid-evidence)

## Source files

- [People.fpas](../../../../examples/ui/gui/gallery/Pages/DataGrid/People.fpas)
- [State.fpas](../../../../examples/ui/gui/gallery/Pages/DataGrid/State.fpas)
- [Update.fpas](../../../../examples/ui/gui/gallery/Pages/DataGrid/Update.fpas)
- [grid_regressions.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/grid_regressions.c)
