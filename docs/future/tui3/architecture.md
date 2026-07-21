# Std.Tui3 architecture

## Unit boundary

`Std.Tui3` is the temporary public unit. Implementation splits into focused private FPAS
units under `lib/Std/Tui3/`. Nested directories introduce matching namespace segments.

Each source file owns one concern. Value records normally use one record type per file
(for example `Geometry/Point.fpas`, `Geometry/Size.fpas`, `Geometry/Rect.fpas`).

`Std.Tui3.fpas` stays a small public facade. It exposes the supported API and delegates to
implementation units. User code imports `Std.Tui3` only; it must not import private units
merely because they ship with the standard library.

After promote, the public unit becomes `Std.Tui` and the directory becomes `lib/Std/Tui/`.
The `Tui` type prefix does not change.

## Public names

All public TUI types use the `Tui` prefix:

| Type | Purpose |
| --- | --- |
| `TuiPoint` / `TuiSize` / `TuiRect` | Cell geometry. |
| `TuiMsg` | Framework messages delivered to `Update`. |
| `TuiCmd` | Deferred effects requested by `Update`. |
| `TuiAction` | Positive integer identity emitted by interactive elements. |
| `TuiElement` | Immutable description of one UI node (and its children). |
| `TuiApplication` | Headless or interactive MVU host (run loop ownership). |

`Std.Console.Rect` remains console-absolute. `TuiRect` is separate so local layout geometry
cannot be confused with screen operations.

## Layers

```text
application Model + Update + View
        ↓
   TuiElement tree (data)
        ↓
   pure layout (rects)
        ↓
   TV-skin paint → TuiSurface
        ↓
   Std.Console cells (interactive only)
```

| Layer | Responsibility |
| --- | --- |
| Application | Owns `Model`. Implements `Update` and `View`. Maps `TuiAction` ids to intent. |
| MVU runtime | Runs the loop, turns console/input into `TuiMsg`, applies `TuiCmd`, calls `View`. |
| Elements | Pure constructors describing chrome and controls. |
| Layout | Pure measure/arrange over element trees. |
| Paint | Draws Turbo Vision-looking frames, text, and controls into a cell surface. |
| Terminal | Acquires modes; copies the surface; reads events. No widget logic. |

## What is not public

- Generational view registries and live handle APIs from Tui2.
- Widget-owned event properties (`OnClick`, `OnChanged`, attach/detach hooks).
- Imperative modal open/close on dialog objects.
- Rust `turbo-vision` types or bridge handles.
- Ambient “current application” constructors.

## Type-owned construction

Value and element construction uses static record functions where they fit
(`[record methods](../../pascal/language/types/record-methods.md)`). Element helpers may
also be free functions on the facade when that reads more clearly for tree building
(`Tui.Desktop([...])` style). Tui3 does not require function overloading.

## Relationship to earlier attempts

| Unit | Role after Tui3 exists |
| --- | --- |
| `Std.Tui` | Turbo Vision crate bridge — delete on promote. |
| `Std.Tui2` | Retained FPAS experiment — frozen; salvage values only; delete on promote. |
| `Std.Tui3` | Current plan and temporary implementation unit. |

See [mvu.md](mvu.md) and [elements.md](elements.md) for the programming model details.
