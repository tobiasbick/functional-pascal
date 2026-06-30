# Std.Tui cell width

Unicode display-width policy for terminal-cell layout. The helpers live in `fpas-std` for future Turbo Vision title and label rendering; they are not part of the current public Pascal API.

The public test surface exposes rendered output through:

| Symbol | Description |
| --- | --- |
| `Application.QueryScreenLine(App, Y): string` | Return one rendered line. |
| `Application.QueryScreenCell(App, X, Y): ScreenCell` | Return one rendered cell. |

`ScreenCell.ch` stores the rendered cell text, and `ScreenCell.fg` / `ScreenCell.bg` store color indexes.

## See Also

- [Application](app/README.md)
- [Types](app/types.md)
