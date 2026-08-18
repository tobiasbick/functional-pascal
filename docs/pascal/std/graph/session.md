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

`Std.Graph` provides native-window graphics with **hosted dispatch** (see [Hosted dispatch](app/README.md)):

- one native window per process
- `Application.Configure` + `Application.Run` with `On*` handlers
- drawing via runtime backbuffer, bulk `UploadFrame`, or immediate-mode helpers inside `OnPaint`
- shared internal event normalization via `UiHost` / `UiEvent` in Rust

Current runtime constraints:

- `Std.Graph.Application.*` is main-task only; do not call it from `go` tasks
- `DrawText` uses a built-in deterministic bitmap font
- the runtime uses `winit` + `softbuffer`
- transient native `0x0` resize callbacks are ignored; `Size` keeps the last positive drawable extent
- `Application.Open` rejects surfaces larger than `64 * 1024 * 1024` pixels (`Width * Height`)

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `Application` | opaque graphics-session handle |
| type | `Size` | record with `width` and `height` |
| type | `EventKind` | `CloseRequested`, `Resize`, `Key`, `Mouse`, `Wheel` |
| type | `Event` | event record with size, key, mouse, wheel, and modifier fields |
| type | `ApplicationHandlers` | bundled hosted handler registration |
| type | `ExitReason` | why `Application.Run` stopped |
| function | `Application.Open(Width: integer; Height: integer; Title: string): Application` | open one native window |
| procedure | `Application.Close(App: Application)` | close the session |
| procedure | `Application.Configure(App: Application; Handlers: ApplicationHandlers)` | register hosted handlers |
| procedure | `Application.Run(App: Application)` | hosted main loop |
| function | `Application.Size(App: Application): Size` | current drawable size |
| procedure | `Application.RequestRedraw(App: Application)` | request `OnPaint` |
| procedure | `Application.HostRequestQuit(App: Application)` | cooperative quit |
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

## Dispatch model

Full applications use hosted dispatch — see [Hosted dispatch](app/README.md) for handler signatures, `ExitReason`, and VM bridge details.

Sample: [`examples/pascal/std/graph_basics.fpas`](../../../../examples/pascal/std/graph_basics.fpas)

---

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

Close the active graphics session. Closing an already closed session is a
no-op. Backend teardown detaches the session before reporting a host close
error, so a failed close does not leave an orphaned backend that blocks a later
`Application.Open`. Runtime teardown also releases an open session
best-effort when execution ends without an explicit close.

### `function Application.Size(App: Application): Size`

Return the latest known drawable size.

Transient native `0x0` resize callbacks are ignored, so `Size` continues to report the last positive drawable extent.

### `procedure Application.UploadFrame(App: Application; Width: integer; Height: integer; Pixels: array of integer)`

Validate and present one full row-major framebuffer.

- pixels use packed `$00RRGGBB`
- `Length(Pixels)` must equal `Width * Height`
- `Width` and `Height` must match the current drawable size
- transient native `0x0` resize callbacks do not change the expected upload extent

If the window is resized again after one earlier `Application.Size(App)` observation, a frame built for that last observed size is still accepted instead of aborting the program. The next size observation or resize handler updates the expected extent.

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

See [examples/pascal/std/graph_basics.fpas](../../../../examples/pascal/std/graph_basics.fpas) for a complete smoke example.

```pascal
uses Std.Graph;

procedure OnPaint(App: Application);
begin
  var Screen: Size := Application.Size(App);
  Application.Clear(App, $00020408);
  Application.DrawText(App, 2, 2, 'FPAS', $00FFFFFF)
end;

begin
  var App: Application := Application.Open(32, 24, 'Graph basics');
  Application.Configure(App, record OnPaint := OnPaint end);
  Application.Run(App)
end.
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime logic | [crates/fpas-std/src/graph/](../../../../crates/fpas-std/src/graph/mod.rs) |
| Type checking | [crates/fpas-sema/src/std_registry/loaded/graph/](../../../../crates/fpas-sema/src/std_registry/loaded/graph/mod.rs) |
| Compiler intrinsic catalog | [crates/fpas-compiler/src/intrinsic_catalog.rs](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| VM bridge | [crates/fpas-vm/src/vm/hosted/graph/](../../../../crates/fpas-vm/src/vm/hosted/graph/mod.rs) |
| Intrinsics | [crates/fpas-bytecode/src/intrinsic/graph.rs](../../../../crates/fpas-bytecode/src/intrinsic/graph.rs) |

## See also

- [Graphics index](README.md)
- [Hosted dispatch](app/README.md)
- [Standard library index](../README.md)
