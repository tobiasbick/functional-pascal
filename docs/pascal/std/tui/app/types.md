# Std.Tui types

## `Application`

Opaque handle for a TUI application session.

## `Rect`

Rectangle in terminal cells.

| Field | Type | Meaning |
| --- | --- | --- |
| `x` | `integer` | Left edge. |
| `y` | `integer` | Top edge. |
| `width` | `integer` | Width in cells. |
| `height` | `integer` | Height in cells. |

## `Size`

| Field | Type | Meaning |
| --- | --- | --- |
| `width` | `integer` | Width in cells. |
| `height` | `integer` | Height in cells. |

## `TuiDialog`

Opaque Turbo Vision dialog handle returned by `Application.CreateDialog`.

## `TuiButton`

Opaque Turbo Vision button handle returned by `Application.CreateButton`.

## `ScreenCell`

| Field | Type | Meaning |
| --- | --- | --- |
| `ch` | `string` | Cell text. |
| `fg` | `integer` | Foreground color. |
| `bg` | `integer` | Background color. |

## `ApplicationHandlers`

Record for bundled transition handlers used by `Application.Configure`. Optional handler fields use `Some(Handler)` or `None`.

For new Turbo Vision command dispatch, prefer `Application.OnCommand(App, Handler)`.

## `ExitReason`

Registered enum for transition run-loop exit reporting.

| Variant | Meaning |
| --- | --- |
| `UserQuit` | The application requested exit. |
| `HostStop` | The backend stopped the active run. |
| `HostAndUserStop` | Both stop paths happened during the same run. |
| `HostShutdown` | The VM entered shutdown while the run was active. |

## See Also

- [Application](README.md)
- [Session API](../session.md)
