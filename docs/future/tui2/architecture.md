# Std.Tui2 architecture

## Unit boundary

`Std.Tui2` is the public unit. Its implementation may be split into focused private FPAS units below `lib/Std/Tui2/`.

The source standard-library loader must distinguish public units from private implementation units. User code may import `Std.Tui2`; it must not import internal Tui2 units merely because their source files are bundled.

The manifest and trusted namespace rules are fixed in [source-library.md](source-library.md).

## Public names

All public TUI types use the `Tui` prefix:

| Type | Purpose |
| --- | --- |
| `TuiPoint` | A cell coordinate. |
| `TuiSize` | A width and height in cells. |
| `TuiRect` | A rectangle used for local view bounds and clipping. |
| `TuiEvent` | The event representation routed by the application. |
| `TuiApplication` | Application lifecycle, dispatch, and redraw ownership. |
| `TuiView` | An opaque live view identity. |
| `TuiCommand` | A command identifier. |

`Std.Console.Rect` remains the type for console-level, screen-absolute cell operations. `TuiRect` is separate so local view geometry cannot be confused with screen operations.

## View model

Std.Tui2 does not emulate the Turbo Vision object hierarchy. FPAS has records and functions rather than classical inheritance.

The library stores live view state in an internal registry. Public values such as `TuiView` and `TuiApplication` are opaque identities. Operations are explicit functions, for example `TuiView.SetBounds`, `TuiDesktop.Add`, and `TuiApplication.Invalidate`.

The registry is the single source of truth for bounds, visibility, enabled state, parent-child ownership, z-order, focusability, pointer capture, and modal ownership.

The planned records, handles, controls, and rough operations are inventoried in [api-surface.md](api-surface.md).

Handle capabilities and teardown are defined in [handles-and-ownership.md](handles-and-ownership.md). Geometry and coordinate conversion are defined in [geometry.md](geometry.md).
