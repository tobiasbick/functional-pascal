# `Std.Graph`

Native windowed 2D drawing for Functional Pascal.

Add the unit to your program:

```pascal
program Example;
uses Std.Graph;
begin
  var App: Application := Application.Open(320, 200, 'Graph');
  Application.Close(App)
end.
```

**Maintenance (implementers only):** keep this file aligned with [crates/fpas-sema/src/std_registry/loaded/graph/mod.rs](../../../crates/fpas-sema/src/std_registry/loaded/graph/mod.rs), [crates/fpas-sema/src/std_registry/loaded/graph/application_api.rs](../../../crates/fpas-sema/src/std_registry/loaded/graph/application_api.rs), [crates/fpas-compiler/src/compiler/std_calls/graph.rs](../../../crates/fpas-compiler/src/compiler/std_calls/graph.rs), [crates/fpas-vm/src/vm/execute/io/graph/](../../../crates/fpas-vm/src/vm/execute/io/graph/mod.rs), [crates/fpas-std/src/graph/](../../../crates/fpas-std/src/graph/mod.rs), and [crates/fpas-bytecode/src/intrinsic/graph.rs](../../../crates/fpas-bytecode/src/intrinsic/graph.rs).

---

## Importing and names

After `uses Std.Graph;` you can use either fully qualified names such as `Std.Graph.Application.Open(...)` or short names such as `Application.Open(...)`.

`Std.Graph` reuses several input types from `Std.Console`:

- `Event.key` has type `Std.Console.KeyEvent`
- `Event.mouse_action` has type `Std.Console.MouseAction`
- `Event.mouse_button` has type `Std.Console.MouseButton`

Import `Std.Console` alongside `Std.Graph` when you want short names such as `KeyKind`, `MouseAction`, `MouseButton`, or `WriteLn`.

If a file also references console-side names such as `Std.Console.KeyKind.*`, `Std.Console.MouseAction.*`, or `Std.Console.MouseButton.*`, keep those console names qualified.

---

## Current status

`Std.Graph` currently provides the first native-window graphics path in FPAS:

- one native window per process
- a bulk `UploadFrame` path for software renderers
- a runtime-owned backbuffer with pixels, lines, rectangles, circles, text, and `Present`
- `PollEvent` support for `CloseRequested`, `Resize`, `Key`, `Mouse`, and `Wheel`

Current runtime constraints:

- `Std.Graph.Application.*` is main-task only; do not call it from `go` tasks
- `DrawText` uses a built-in deterministic bitmap font
- the runtime currently targets `winit` + `softbuffer`

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Application` | opaque graphics-session handle |
| type | `Size` | record with `width` and `height` |
| type | `EventKind` | `CloseRequested`, `Resize`, `Key`, `Mouse`, `Wheel` |
| type | `Event` | event record with size, key, mouse, wheel, and modifier fields |
| function | `Application.Open(Width: integer; Height: integer; Title: string): Application` | open one native window |
| procedure | `Application.Close(App: Application)` | close the session |
| function | `Application.Size(App: Application): Size` | current drawable size |
| function | `Application.PollEvent(App: Application): Option of Event` | non-blocking event poll |
| procedure | `Application.UploadFrame(App: Application; Width: integer; Height: integer; Pixels: array of integer)` | bulk row-major `$00RRGGBB` upload |
| procedure | `Application.Clear(App: Application; Color: integer)` | fill the runtime backbuffer |
| procedure | `Application.PutPixel(App: Application; X: integer; Y: integer; Color: integer)` | write one clipped pixel |
| procedure | `Application.DrawLine(App: Application; X1: integer; Y1: integer; X2: integer; Y2: integer; Color: integer)` | draw one clipped line |
| procedure | `Application.DrawRect(App: Application; X: integer; Y: integer; Width: integer; Height: integer; Color: integer)` | draw one clipped rectangle outline |
| procedure | `Application.FillRect(App: Application; X: integer; Y: integer; Width: integer; Height: integer; Color: integer)` | fill one clipped rectangle |
| procedure | `Application.DrawCircle(App: Application; CenterX: integer; CenterY: integer; Radius: integer; Color: integer)` | draw one clipped circle outline |
| procedure | `Application.DrawText(App: Application; X: integer; Y: integer; Text: string; Color: integer)` | draw deterministic bitmap text |
| procedure | `Application.Present(App: Application)` | flush the runtime backbuffer |

---

## Types

### Type `Application`

Logical name: `Std.Graph.Application`. Short: `Application` when `Std.Graph` is imported.

`Application` is an opaque handle for one native graphics session.

---

### Type `Size`

Logical name: `Std.Graph.Size`. Short: `Size` when `Std.Graph` is imported.

Conceptual declaration:

```pascal
type Size = record
  width: integer;
  height: integer
end;
```

---

### Type `EventKind`

Logical name: `Std.Graph.EventKind`. Short: `EventKind` when `Std.Graph` is imported.

Variants:

- `CloseRequested`
- `Resize`
- `Key`
- `Mouse`
- `Wheel`

---

### Type `Event`

Logical name: `Std.Graph.Event`. Short: `Event` when `Std.Graph` is imported.

Conceptual declaration:

```pascal
type Event = record
  kind: EventKind;
  size: Size;
  key: Std.Console.KeyEvent;
  mouse_action: Std.Console.MouseAction;
  mouse_button: Std.Console.MouseButton;
  mouse_x: integer;
  mouse_y: integer;
  wheel_x: integer;
  wheel_y: integer;
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean
end;
```

Field usage by event kind:

- `CloseRequested`: only `kind` is meaningful
- `Resize`: `size` is populated
- `Key`: `key` is populated; modifier booleans mirror the key event
- `Mouse`: `mouse_action`, `mouse_button`, `mouse_x`, `mouse_y`, and modifiers are populated
- `Wheel`: `wheel_x`, `wheel_y`, `mouse_x`, `mouse_y`, and modifiers are populated

Mouse coordinates use the same 0-based pixel space as the drawing API.

---

## Routines

### `function Application.Open(Width: integer; Height: integer; Title: string): Application`

Open one native graphics window.

- `Width` and `Height` must be positive.
- `Title` becomes the initial window title.

### `procedure Application.Close(App: Application)`

Close the active graphics session. Closing an already closed session is a no-op.

### `function Application.Size(App: Application): Size`

Return the latest known drawable size.

### `function Application.PollEvent(App: Application): Option of Event`

Return `Some(E)` when one event is queued, or `None` when no event is pending.

### `procedure Application.UploadFrame(App: Application; Width: integer; Height: integer; Pixels: array of integer)`

Validate and present one full row-major framebuffer.

- pixels use packed `$00RRGGBB`
- `Length(Pixels)` must equal `Width * Height`
- `Width` and `Height` must match the current drawable size

### `procedure Application.Clear(App: Application; Color: integer)`

Fill the runtime-owned backbuffer with one packed `$00RRGGBB` color.

### `procedure Application.PutPixel(App: Application; X: integer; Y: integer; Color: integer)`

Write one pixel into the runtime-owned backbuffer. Out-of-bounds coordinates are clipped.

### `procedure Application.DrawLine(App: Application; X1: integer; Y1: integer; X2: integer; Y2: integer; Color: integer)`

Draw one clipped line into the runtime-owned backbuffer.

### `procedure Application.DrawRect(App: Application; X: integer; Y: integer; Width: integer; Height: integer; Color: integer)`

Draw one clipped rectangle outline. `Width` and `Height` must be positive.

### `procedure Application.FillRect(App: Application; X: integer; Y: integer; Width: integer; Height: integer; Color: integer)`

Fill one clipped rectangle. `Width` and `Height` must be positive.

### `procedure Application.DrawCircle(App: Application; CenterX: integer; CenterY: integer; Radius: integer; Color: integer)`

Draw one clipped circle outline. `Radius` must be non-negative.

### `procedure Application.DrawText(App: Application; X: integer; Y: integer; Text: string; Color: integer)`

Draw deterministic bitmap text into the runtime-owned backbuffer.

### `procedure Application.Present(App: Application)`

Flush the current runtime-owned backbuffer to the native window.

---

## Example

See [examples/pascal/std/graph_basics.fpas](../../../examples/pascal/std/graph_basics.fpas) for a complete smoke example and [examples/math/julia/julia_graph.fpas](../../../examples/math/julia/julia_graph.fpas) for an interactive explorer.

```pascal
uses Std.Console, Std.Conv, Std.Graph, Std.Option;

var App: Application := Application.Open(32, 24, 'Graph basics');
var Screen: Size := Application.Size(App);

Application.Clear(App, $00020408);
Application.DrawRect(App, 0, 0, Screen.width, Screen.height, $0000C080);
Application.DrawLine(App, 0, 0, Screen.width - 1, Screen.height - 1, $00FF8040);
Application.DrawCircle(App, Screen.width div 2, Screen.height div 2, 5, $0040A0FF);
Application.DrawText(App, 2, 2, 'FPAS', $00FFFFFF);
Application.Present(App);

WriteLn('size=', IntToStr(Screen.width), 'x', IntToStr(Screen.height));
WriteLn('pending=', BoolToStr(Std.Option.IsSome(Application.PollEvent(App))));

Application.Close(App)
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|----------|
| Runtime logic | [crates/fpas-std/src/graph/](../../../crates/fpas-std/src/graph/mod.rs) |
| Type checking | [crates/fpas-sema/src/std_registry/loaded/graph/](../../../crates/fpas-sema/src/std_registry/loaded/graph/mod.rs) |
| Compiler lowering | [crates/fpas-compiler/src/compiler/std_calls/graph.rs](../../../crates/fpas-compiler/src/compiler/std_calls/graph.rs) |
| VM bridge | [crates/fpas-vm/src/vm/execute/io/graph/](../../../crates/fpas-vm/src/vm/execute/io/graph/mod.rs) |
| Intrinsics | [crates/fpas-bytecode/src/intrinsic/graph.rs](../../../crates/fpas-bytecode/src/intrinsic/graph.rs) |

[← Standard library index](README.md)