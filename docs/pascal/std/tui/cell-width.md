# Std.Tui cell width

Unicode display-width policy for terminal-cell layout. The helpers live in `fpas-std` for Turbo Vision title and label rendering; they are not part of the current public Pascal API.

Headless tests assert rendered output through [`Std.Test`](../../testing/test.md) on the shared console back buffer:

| Symbol | Description |
| --- | --- |
| `AssertScreenLine(Expected, Y)` | Compare one rendered row (`Y` is one-based). |
| `AssertScreenCell(X, Y, Ch, Fg, Bg)` | Compare one CRT cell character and packed colors. |

Use `uses Std.Console` alongside `Std.Tui` when calling these helpers from Turbo Vision tests.

## See Also

- [Application](app/README.md)
- [Native testing](app/testing.md)
