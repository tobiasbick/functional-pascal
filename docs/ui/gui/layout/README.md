# Layout

Layout shows how CUI layers children, mirrors a row and scrolls overflowing
content. Use layout constraints to give the containers the space they need.

| Topic | When to use it |
| --- | --- |
| [Layered stack](stack.md) | Place children in the same area and control their paint order. |
| [Divider](divider.md) | Separate sections with a line and an optional caption. |
| [Reading direction and spacing](reading-direction.md) | Mirror a row with logical padding, a fixed gap and a spacer. |
| [Scroll view](scroll-view.md) | Clip overflowing content and let the user scroll it. |

## See also

- [UI handbook](../README.md)
- [Layout in the gallery](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [Grid layout](../feedback/grid.md)
- [FPAS layout reference](../../widgets.md#native-graphical-layout)

## Source files

- [layout_parity.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
- [LayoutContract.fpas](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
- [CuiContainerTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiContainerTests.cs)
- [CuiScrollRevealTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiScrollRevealTests.cs)
