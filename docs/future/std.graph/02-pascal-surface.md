# Proposed `Std.Graph` Pascal surface

**Status:** draft for the first public slice.

## Design intent

`Std.Graph` should start with a small, explicit surface that matches the Phase 1 MVP.
It should expose a native window and a bulk framebuffer upload path first.
That foundation should then grow into a small modern 2D drawing API with direct input handling.

## Naming decisions

- [x] `UploadFrame` is the bulk pixel-upload fast path.
- [x] `Present` is reserved for presenting the runtime-owned backbuffer after drawing calls.
- [x] Event kinds use short names such as `Resize` and `Key` to stay aligned with existing `Std.Tui` and `Std.Console` event naming.

## Target capability set

The intended direction is:

- [ ] bulk software frame upload for render-heavy programs
- [ ] a runtime-owned backbuffer for drawing primitives
- [ ] pixels, lines, simple shapes, and text
- [ ] keyboard, mouse, wheel, resize, and close events
- [x] enough control to build Mandelbrot and Julia explorers

## Proposed types

```pascal
type
  Application = record end;

  Size = record
    width: integer;
    height: integer;
  end;

  EventKind = (
    CloseRequested,
    Resize,
    Key,
    Mouse,
    Wheel
  );

  Event = record
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
    meta: boolean;
  end;
```

## Proposed routines

```pascal
function Application.Open(Width: integer; Height: integer; Title: string): Application;
procedure Application.Close(App: Application);
function Application.Size(App: Application): Size;
function Application.PollEvent(App: Application): Option of Event;
procedure Application.UploadFrame(
  App: Application;
  Width: integer;
  Height: integer;
  Pixels: array of integer
);
procedure Application.Clear(App: Application; Color: integer);
procedure Application.PutPixel(App: Application; X: integer; Y: integer; Color: integer);
procedure Application.DrawLine(
  App: Application;
  X1: integer;
  Y1: integer;
  X2: integer;
  Y2: integer;
  Color: integer
);
procedure Application.DrawRect(
  App: Application;
  X: integer;
  Y: integer;
  Width: integer;
  Height: integer;
  Color: integer
);
procedure Application.FillRect(
  App: Application;
  X: integer;
  Y: integer;
  Width: integer;
  Height: integer;
  Color: integer
);
procedure Application.DrawCircle(
  App: Application;
  CenterX: integer;
  CenterY: integer;
  Radius: integer;
  Color: integer
);
procedure Application.DrawText(
  App: Application;
  X: integer;
  Y: integer;
  Text: string;
  Color: integer
);
procedure Application.Present(App: Application);
```

## Current richer input surface

- [x] `EventKind.Mouse` covers down, up, move, and drag through `Event.mouse_action`.
- [x] `EventKind.Wheel` carries signed `wheel_x` / `wheel_y` deltas.
- [x] Mouse positions use the same 0-based pixel coordinate space as the drawing API.
- [x] `Event.mouse_action` and `Event.mouse_button` reuse `Std.Console.MouseAction` and `Std.Console.MouseButton`.

## Semantics

### `Application.Open`

- [ ] Opens one native window.
- [ ] `Width` and `Height` must be positive.
- [ ] `Title` becomes the initial window title.
- [ ] Phase 1 supports a single active graphics application per process.

### `Application.Close`

- [ ] Releases the window and associated host resources.
- [ ] Closing an already closed application is a no-op.

### `Application.Size`

- [ ] Returns the latest known drawable size.
- [ ] Width and height are always positive after a successful `Open`.

### `Application.PollEvent`

- [ ] Returns `None` when no event is pending.
- [ ] `Event.kind = Resize` populates `Event.size`.
- [ ] `Event.kind = Key` populates `Event.key`.
- [ ] `Event.kind = Mouse` populates `Event.mouse_action`, `Event.mouse_button`, `Event.mouse_x`, `Event.mouse_y`, and modifier flags.
- [ ] `Event.kind = Wheel` populates `Event.wheel_x`, `Event.wheel_y`, `Event.mouse_x`, `Event.mouse_y`, and modifier flags.
- [ ] `Event.kind = CloseRequested` signals that the host asked the application to exit.

### `Application.UploadFrame`

- [ ] `Pixels` is a row-major framebuffer.
- [ ] Each pixel is encoded as `$00RRGGBB`.
- [ ] `Length(Pixels)` must equal `Width * Height`.
- [ ] Phase 1 requires `Width` and `Height` to match the current window size exactly.
- [ ] Any mismatch should raise a clear runtime diagnostic that reports the expected size.

### `Application.Clear`

- [ ] Fills the runtime-owned backbuffer with one packed `$00RRGGBB` color.

### `Application.PutPixel`

- [ ] Writes one pixel into the runtime-owned backbuffer.
- [ ] Out-of-bounds coordinates are clipped.

### `Application.DrawLine`

- [ ] Draws one line into the runtime-owned backbuffer.
- [ ] Coordinates are clipped to the current framebuffer bounds.

### `Application.DrawRect`

- [ ] Draws one rectangle outline into the runtime-owned backbuffer.
- [ ] `Width` and `Height` must be positive.

### `Application.FillRect`

- [ ] Fills one rectangle into the runtime-owned backbuffer.
- [ ] `Width` and `Height` must be positive.

### `Application.DrawCircle`

- [ ] Draws one circle outline into the runtime-owned backbuffer.
- [ ] `Radius` must be non-negative.

### `Application.DrawText`

- [ ] Draws deterministic bitmap text into the runtime-owned backbuffer.
- [ ] Each glyph uses a fixed-size built-in bitmap font.
- [ ] Drawing is clipped to the current framebuffer bounds.

### `Application.Present`

- [ ] Flushes the current runtime-owned backbuffer to the native window.

## Planned later semantics for drawing routines

- [x] Drawing routines mutate a runtime-owned backbuffer.
- [x] `Application.Present(App)` flushes that backbuffer to the native window.
- [x] `Application.UploadFrame` remains the bulk upload fast path for render-heavy programs.
- [x] The runtime clips drawing operations to the current framebuffer bounds.
- [x] `Application.DrawText(App, X, Y, Text, Color)` now uses a simple deterministic bitmap font.

## Minimal example

```pascal
program GraphSmoke;
uses Std.Console, Std.Graph;

function MakeSolidFrame(Width: integer; Height: integer; Color: integer): array of integer;
begin
  var Pixels: array of integer := [];
  var Count: integer := Width * Height;
  var I: integer := 0;
  while I < Count do
  begin
    Pixels := Pixels + [Color];
    I := I + 1
  end;
  return Pixels
end;

begin
  var App: Application := Application.Open(640, 480, 'Graph smoke');
  var Running: boolean := true;

  while Running do
  begin
    var S: Size := Application.Size(App);
    var Frame: array of integer := MakeSolidFrame(S.width, S.height, $00102040);
    Application.UploadFrame(App, S.width, S.height, Frame);

    match Application.PollEvent(App) with
      Some(E) =>
        if E.kind = EventKind.CloseRequested then
          Running := false
        else if E.kind = EventKind.Key then
          if E.key.kind = KeyKind.Escape then
            Running := false;
      None =>
        begin end
    end
  end;

  Application.Close(App)
end.
```

## Deliberately deferred surface

The following should stay out of Phase 1 even if they are desired long-term:

- drawing primitives listed above
- mouse events
- image loading helpers
- multiple window APIs

## Open design note

Phase 1 reuses `Std.Console.KeyEvent` to avoid inventing a second keyboard abstraction immediately.
If later graphics work needs richer window-system keyboard data, `Std.Graph` can define its own key event record in a later phase.