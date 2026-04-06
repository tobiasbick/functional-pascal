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

---

## Current status

`Std.Tui` currently defines the **semantic surface** for the first application model:

- `Application` is the TUI session handle.
- `Size` exposes terminal width and height.
- `Event` exposes key and resize input.
- `Application.RequestRedraw` / `Application.RedrawPending` support redraw-oriented loops.

This surface is intended for semantic analysis and API design. Runtime bindings for the unit are not part of the current implementation yet.

---

## Quick reference

Everything below requires `uses Std.Tui;`.

| Kind | Name | Notes |
|------|------|--------|
| type | `Application` | opaque application/session handle |
| type | `Size` | record with `width` and `height` |
| type | `KeyKind` | enum for logical keys |
| type | `KeyEvent` | record for one key input |
| type | `EventKind` | enum with `Key` and `Resize` |
| type | `Event` | record for one application event |
| function | `Application.Open(): Application` | create/open an application session |
| procedure | `Application.Close(App: Application)` | close the application session |
| function | `Application.Size(App: Application): Size` | current terminal size |
| function | `Application.ReadEvent(App: Application): Event` | blocking event read |
| function | `Application.ReadEventTimeout(App: Application; Milliseconds: integer): Option of Event` | wait up to N ms |
| function | `Application.PollEvent(App: Application): Option of Event` | non-blocking event check |
| procedure | `Application.RequestRedraw(App: Application)` | mark the application as needing redraw |
| function | `Application.RedrawPending(App: Application): boolean` | query redraw state |
| enum members | `KeyKind.Space`, `KeyKind.Escape`, `KeyKind.Character`, … | same logical key set as the console key model |
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

### Type `KeyKind`

Logical name: `Std.Tui.KeyKind`. Short: `KeyKind` when `Std.Tui` is imported.

`KeyKind` uses the same logical key set as the terminal key model already used elsewhere in the standard library:

- `Unknown`
- `Escape`
- `Tab`
- `Enter`
- `Backspace`
- `Space`
- `Up`
- `Down`
- `Left`
- `Right`
- `Home`
- `End`
- `PageUp`
- `PageDown`
- `Insert`
- `Delete`
- `F1` .. `F12`
- `Character`

---

### Type `KeyEvent`

Logical name: `Std.Tui.KeyEvent`. Short: `KeyEvent` when `Std.Tui` is imported.

Conceptual declaration:

```pascal
type KeyEvent = record
  kind: KeyKind;
  ch: char;
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean
end;
```

---

### Type `EventKind`

Logical name: `Std.Tui.EventKind`. Short: `EventKind` when `Std.Tui` is imported.

Variants:

- `Key`
- `Resize`

---

### Type `Event`

Logical name: `Std.Tui.Event`. Short: `Event` when `Std.Tui` is imported.

Conceptual declaration:

```pascal
type Event = record
  kind: EventKind;
  key: KeyEvent;
  size: Size
end;
```

| Field | Type | Meaning |
|-------|------|---------|
| `kind` | `EventKind` | which event payload is active |
| `key` | `KeyEvent` | populated for `EventKind.Key` |
| `size` | `Size` | populated for `EventKind.Resize` |

---

## Routines

### `function Application.Open(): Application`

Create or open a terminal application session.

The intended model is that this acquires the terminal session needed for a TUI loop.

### `procedure Application.Close(App: Application)`

Close the application session and release its terminal-session ownership.

### `function Application.Size(App: Application): Size`

Return the current terminal size for the application.

### `function Application.ReadEvent(App: Application): Event`

Read one event from the application event stream.

### `function Application.ReadEventTimeout(App: Application; Milliseconds: integer): Option of Event`

Wait up to `Milliseconds` for an event. Returns `None` if no event is available before the timeout.

### `function Application.PollEvent(App: Application): Option of Event`

Return the next event if one is already available; otherwise return `None`.

### `procedure Application.RequestRedraw(App: Application)`

Mark the application as needing a redraw.

### `function Application.RedrawPending(App: Application): boolean`

Return `true` when the application should render a new frame.

---

## Example loop

```pascal
program Demo;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.RequestRedraw(App);

  while true do
  begin
    case Application.PollEvent(App) of
      Some(E):
        if E.kind = EventKind.Resize then
          Application.RequestRedraw(App);
      None:
        begin
        end
    end;

    if Application.RedrawPending(App) then
    begin
      { update local state and render here }
    end
  end
end.
```
