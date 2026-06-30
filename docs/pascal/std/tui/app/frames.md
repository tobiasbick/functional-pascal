# Std.Tui frame transition API

The old public `Application.Host*` frame mutators are no longer registered.

These frame symbols remain during the Turbo Vision migration:

| Symbol | Description |
| --- | --- |
| `Application.ShowFramedDialog(App, ModalId, X, Y, Width, Height, Title, Movable, Resizable, Zoomable, Scrollable, Closable): ViewId` | Create and show a transition framed dialog. |
| `Application.QueryFrameRootState(App, ViewId): FrameRootState` | Read transition frame geometry and flags. |
| `Application.QueryFrameScrollState(App, ViewId): FrameScrollState` | Read transition frame scroll state. |
| `Application.QueryFrameWindowList(App): array of FrameWindowEntry` | Read transition window entries. |

These symbols are temporary bridge surface. New code should prefer the Turbo Vision handle API in [Application](README.md).

## See Also

- [Application](README.md)
- [Types](types.md)
