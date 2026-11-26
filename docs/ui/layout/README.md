# UI layout

`Fpas.Ui` defines layout for views that run in the GUI or TUI host. Use a host-specific option when the view needs behavior unique to that host.

| Topic | Use |
| --- | --- |
| [Sizing and units](sizing.md) | Choose content, fill, fixed or percentage sizing. Set bounds and share free space. |
| [Responsive views](responsive.md) | Choose a layout from the viewport size or its width and height classes. |
| [Scroll position](scroll-position.md) | Choose who owns a scroll offset and handle scroll proposals. |

## Composition

Use `Elements.Row` to arrange children horizontally and `Elements.Column` to arrange them vertically. The direction of the container defines its main axis. The perpendicular axis is its cross axis.

Pass `Layout.Flow` to set the gap, padding and cross-axis alignment. Both constructors use zero gap and zero padding when you omit `Flow`. Default alignment centers children in a row and stretches them across a column. A widget can have its own alignment or size.

Use `Elements.Size` to set a child's allocation. Use `Elements.Border` to enclose a child with a border and an optional title. Use `Elements.Styled` to apply a role from the host theme.

## Host-specific layout

| Host | Topics |
| --- | --- |
| [GUI layout](../gui/layout/README.md) | CUI box options, wrapping, grids, layers, focus scopes, clipping and finalized bounds. |
| [TUI layout](../tui/layout/README.md) | Cell allocation, panels, viewport dimensions and scrolling. |

## See also

- [UI handbook](../README.md)
- [Capability profiles and style roles](../widgets.md#layout-and-style)

## Source files

- [Layout.fpas](../../../src/Fpas.Ui/Layout.fpas)
- [Elements.fpas](../../../src/Fpas.Ui/Elements.fpas)
- [Styles.fpas](../../../src/Fpas.Ui/Styles.fpas)
