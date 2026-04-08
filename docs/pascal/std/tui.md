# `Std.Tui`

Terminal-application structure for Functional Pascal programs that manage their own state and main loop.

Add the unit to your program:

```pascal
program Example;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.Close(App)
end.
```

**Maintenance (implementers only):** keep this file aligned with [`loaded/tui.rs`](../../../crates/fpas-sema/src/std_registry/loaded/tui.rs) and the standard-unit registry under [`crates/fpas-std/src/std_units/`](../../../crates/fpas-std/src/std_units/mod.rs).

---

## Importing and names

After `uses Std.Tui;` you can refer to the unit in either form:

| Style | Example |
|--------|---------|
| **Fully qualified** | `Std.Tui.Application.Open()` |
| **Short** | `Application.Open()` |

`Std.Tui` exports nested names such as `Application.Open`, `Application.ReadEvent`, and `EventKind.Resize`. These short forms are available only when `Std.Tui` appears in `uses`.

`Std.Tui` builds on [`Std.Console`](console.md): the `key` field of `Std.Tui.TuiEvent` has type **`Std.Console.KeyEvent`** (and its `kind` field is **`Std.Console.KeyKind`**). The **`Tui`** prefix avoids clashing with **`Std.Console.Event`**. Import **`Std.Console`** alongside **`Std.Tui`** when you need short names such as `KeyKind` or `WriteLn`, or use fully qualified `Std.Console.*` names.

---

## Current status

`Std.Tui` currently provides the first **semantic, compiler, and VM-backed** application path:

- `Application` is the TUI session handle.
- `Size` exposes terminal width and height.
- `TuiEvent` exposes key and resize input.
- `Application.RequestRedraw` / `Application.RedrawPending` support redraw-oriented loops.

The initial execution path is intentionally narrow:

- `Application.Open` starts the initial terminal session by owning raw mode and the alternate screen when the runtime is connected to a real terminal.
- `Application.Close` releases that session and restores the terminal state it acquired.
- `Application.Size` reads the current terminal dimensions.
- `Application.ReadEvent`, `Application.ReadEventTimeout`, and `Application.PollEvent` currently surface only `Key` and `Resize` events.
- `Application.RequestRedraw` marks redraw as needed.
- `Application.RedrawPending` reports and clears the pending redraw flag so render loops can consume it once per request.

The broader Rust runtime design for `Std.Tui` is still intentionally minimal and may expand in follow-up work.

---

## Quick reference

Everything below requires `uses Std.Tui;`. Key types for `TuiEvent.key` come from **`Std.Console`** (add `uses Std.Console` for short names like `KeyKind`).

| Kind | Name | Notes |
|------|------|--------|
| type | `Application` | opaque application/session handle |
| type | `Size` | record with `width` and `height` |
| type | `EventKind` | enum with `Key` and `Resize` |
| type | `TuiEvent` | record for one application event (`key` is `Std.Console.KeyEvent`) |
| type | `Std.Console.KeyKind` | enum for logical keys (reused; not defined under `Std.Tui`) |
| type | `Std.Console.KeyEvent` | record for one key input (reused; not defined under `Std.Tui`) |
| function | `Application.Open(): Application` | create/open an application session |
| procedure | `Application.Close(App: Application)` | close the application session |
| function | `Application.Size(App: Application): Size` | current terminal size |
| function | `Application.ReadEvent(App: Application): TuiEvent` | blocking event read |
| function | `Application.ReadEventTimeout(App: Application; Milliseconds: integer): Option of TuiEvent` | wait up to N ms |
| function | `Application.PollEvent(App: Application): Option of TuiEvent` | non-blocking event check |
| procedure | `Application.RequestRedraw(App: Application)` | mark the application as needing redraw |
| function | `Application.RedrawPending(App: Application): boolean` | query redraw state |
| enum members | `Std.Console.KeyKind.*` (short `KeyKind.*` with `uses Std.Console`) | same as [`Std.Console`](console.md) |
| enum members | `EventKind.Key`, `EventKind.Resize` | TUI event kinds |

---

## Types

### Type `Application`

Logical name: `Std.Tui.Application`. Short: `Application` when `Std.Tui` is imported.

`Application` is an opaque handle for terminal-session lifecycle, event access, and redraw coordination. Programs keep ownership of their own state; the handle exists only to model the session itself.

---

### Type `Size`

Logical name: `Std.Tui.Size`. Short: `Size` when `Std.Tui` is imported.

Conceptual declaration:

```pascal
type Size = record
  width: integer;
  height: integer
end;
```

| Field | Type | Meaning |
|-------|------|---------|
| `width` | `integer` | terminal width in cells |
| `height` | `integer` | terminal height in cells |

---

### Key input types (`Std.Console.KeyKind`, `Std.Console.KeyEvent`)

`Std.Tui` does not define its own key types. Use **`Std.Console.KeyKind`** and **`Std.Console.KeyEvent`** for `TuiEvent.key` (see [`console.md`](console.md) — `ReadKeyEvent`, `KeyEvent`, `KeyKind`). With `uses Std.Console`, the short names **`KeyKind`** and **`KeyEvent`** refer to those console types.

---

### Type `EventKind`

Logical name: `Std.Tui.EventKind`. Short: `EventKind` when `Std.Tui` is imported.

Variants:

- `Key`
- `Resize`

---

### Type `TuiEvent`

Logical name: `Std.Tui.TuiEvent`. Short: `TuiEvent` when `Std.Tui` is imported.

Conceptual declaration:

```pascal
type TuiEvent = record
  kind: EventKind;
  key: Std.Console.KeyEvent;
  size: Size
end;
```

| Field | Type | Meaning |
|-------|------|---------|
| `kind` | `EventKind` | which event payload is active |
| `key` | `Std.Console.KeyEvent` | populated for `EventKind.Key` |
| `size` | `Size` | populated for `EventKind.Resize` |

---

## Routines

### `function Application.Open(): Application`

Create or open a terminal application session.

The initial runtime acquires the terminal session needed for a TUI loop by enabling raw mode and entering the alternate screen when the runtime is connected to an interactive terminal.

### `procedure Application.Close(App: Application)`

Close the application session and release its terminal-session ownership.

The initial runtime restores any terminal state acquired by `Application.Open()`.

### `function Application.Size(App: Application): Size`

Return the current terminal size for the application.

### `function Application.ReadEvent(App: Application): TuiEvent`

Read one event from the application event stream.

### `function Application.ReadEventTimeout(App: Application; Milliseconds: integer): Option of TuiEvent`

Wait up to `Milliseconds` for an event. Returns `None` if no event is available before the timeout.

### `function Application.PollEvent(App: Application): Option of TuiEvent`

Return the next event if one is already available; otherwise return `None`.

### `procedure Application.RequestRedraw(App: Application)`

Mark the application as needing a redraw.

### `function Application.RedrawPending(App: Application): boolean`

Return `true` when the application should render a new frame.

---

## Minimal application model

The smallest useful `Std.Tui` flow is:

1. open an application session,
2. request and consume redraws,
3. process events,
4. close the session before exit.

```pascal
program StdTuiMinimal;
uses Std.Console, Std.Tui;
begin
  var App: Application := Application.Open();
  mutable var Running: boolean := true;
  Application.RequestRedraw(App);

  while Running do
  begin
    case Application.ReadEventTimeout(App, 16) of
      Some(E):
        begin
          if E.kind = EventKind.Resize then
            Application.RequestRedraw(App)
          else if E.kind = EventKind.Key then
          begin
            if E.key.kind = Std.Console.KeyKind.Escape then
              Running := false
            else
              Application.RequestRedraw(App)
          end
        end;
      None:
        begin
        end
    end;

    if Application.RedrawPending(App) then
    begin
      var S: Size := Application.Size(App);
      ClrScr();
      GotoXY(1, 1);
      WriteLn('Std.Tui minimal app');
      WriteLn('Size: ', S.width, 'x', S.height);
      WriteLn('Press Escape to exit')
    end
  end;

  Application.Close(App)
end.
```

See also: [`examples/pascal/tui/minimal_application.fpas`](../../../examples/pascal/tui/minimal_application.fpas)
