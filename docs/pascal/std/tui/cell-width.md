# Std.Tui cell width

TUI rendering uses terminal cell widths when text is painted into screen buffers.

The current public API exposes cell contents through:

| Symbol | Description |
| --- | --- |
| `Application.QueryScreenLine(App, Y): string` | Return one rendered line. |
| `Application.QueryScreenCell(App, X, Y): ScreenCell` | Return one rendered cell. |

`ScreenCell.ch` stores the rendered cell text, and `ScreenCell.fg` / `ScreenCell.bg` store color indexes.

## See Also

- [Application](app/README.md)
- [Types](app/types.md)
