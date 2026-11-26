# Terminal layout

The TUI host measures and allocates whole terminal cells. Use the shared
layout API for rows, columns and sizing. Use terminal extensions when a view
needs exact terminal dimensions or terminal shortcuts.

| Topic | When to use it |
| --- | --- |
| [Rows and columns](rows-and-columns.md) | Allocate cells, align children and set gaps and padding. |
| [Panels and weighted layout](panels.md) | Compose a panel from a border, a semantic surface role and a size rule. |
| [Scroll view](scroll-view.md) | Keep a long list or body inside a bounded area. |
| [Terminal viewport](viewport.md) | Adapt the view to a resize or build a subtree from exact cell counts. |

## See also

- [Terminal UI handbook](../README.md)
- [Shared layout](../../layout/README.md)
- [GUI layout](../../gui/layout/README.md)
