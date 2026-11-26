# Layout

FPAS uses `cui_box_begin` for columns, rows, layered stacks and grids.
Use `Layout.Flow` for shared box options and `Elements.Size` to
constrain their dimensions.

| Topic | When to use it |
| --- | --- |
| [Column](column.md) | Arrange children vertically with native alignment, sizing, decoration and scrolling. |
| [Row](row.md) | Arrange children horizontally and wrap them at the available width. |
| [Layered stack](stack.md) | Place children in the same area and control their paint order. |
| [Grid](../feedback/grid.md) | Lay out equal-width columns with a fixed or responsive count. |
| [Divider](divider.md) | Separate sections with a line and an optional caption. |
| [Reading direction and spacing](reading-direction.md) | Mirror a row with logical padding, a fixed gap and a spacer. |
| [Scroll view](scroll-view.md) | Clip overflowing content and let the user scroll it. |

## See also

- [UI handbook](../README.md)
- [Layout in the gallery](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [FPAS layout reference](../../widgets.md#native-graphical-layout)

## Source files

- [layout_parity.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
- [LayoutContract.fpas](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
- [NativeBoxContract.fpas](../../../../tests/Fixtures/gui-host/NativeBoxContract.fpas)
- [CuiContainerTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiContainerTests.cs)
- [CuiScrollRevealTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiScrollRevealTests.cs)
