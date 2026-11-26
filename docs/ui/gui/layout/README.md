# Layout

Use `Layout.Flow` to configure native columns, rows, layered stacks and grids.
Use `Elements.Size` to constrain their dimensions.

## Arrange and size content

| Topic | When to use it |
| --- | --- |
| [Column](column.md) | Arrange children vertically. |
| [Row](row.md) | Arrange children horizontally and wrap them at the available width. |
| [Layered stack](stack.md) | Place children in the same area and control their paint order. |
| [Grid](../feedback/grid.md) | Lay out equal-width columns with a fixed or responsive count. |
| [Card](../feedback/card.md) | Group related content in a surface with its own layout. |
| [Sizing](../../layout/sizing.md) | Constrain one or both axes and choose growth weights. |
| [Responsive layout](../../layout/responsive.md) | Adapt the view to the drawable area. |

## Configure containers

| Topic | When to use it |
| --- | --- |
| [Container options](container-options.md) | Set alignment, clipping, appearance, scrolling and stable IDs. |
| [Spacing](spacing.md) | Set padding, add fixed gaps or distribute free space with spacers. |
| [Divider](divider.md) | Separate sections with a line and an optional caption. |
| [Reading direction](reading-direction.md) | Set the direction of a window, a container or a text element. |
| [Focus scopes](focus-scopes.md) | Limit Tab navigation to the active pane. |
| [Layout inspection](layout-inspection.md) | Read the bounds of named containers and controls from the frame snapshot. |

## Scroll and navigate

| Topic | When to use it |
| --- | --- |
| [Scroll view](scroll-view.md) | Clip overflowing content and let the user scroll it. |
| [Virtual list](../data-display/virtual-list.md) | Build the visible portion of a long list of fixed-height rows. |
| [Scroll anchor](../reader/scroll-anchor.md) | Name a destination inside scrolling content. |
| [Table of contents](../reader/table-of-contents.md) | Jump to a section and track the current heading. |

## Arrange panes and data

| Topic | When to use it |
| --- | --- |
| [Split view](../app-chrome/split-view.md) | Separate two panes with a draggable divider. |
| [Docking workspace](../app-chrome/dockspace.md) | Arrange tabbed panels and save their layout. |
| [Table](../data-display/table.md) | Align text and composed content in rows and columns. |
| [Data grid](../data-grid/data-grid.md) | Arrange many rows with native column sizing and scrolling. |
| [Tree table](../data-grid/tree-table.md) | Align hierarchical rows in columns. |

## See also

- [GUI handbook](../README.md)
- [Shared layout](../../layout/README.md)
- [Layout in the gallery](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [GUI host](../host.md#frame-preparation-and-presentation)

## Source files

- [Layout validation](../../../../src/Fpas.Ui.Gui/Internal/Layout/LayoutValidation.fpas)
- [layout_parity.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
- [LayoutContract.fpas](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
- [NativeBoxContract.fpas](../../../../tests/Fixtures/gui-host/NativeBoxContract.fpas)
- [CuiContainerTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiContainerTests.cs)
- [CuiScrollRevealTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiScrollRevealTests.cs)
