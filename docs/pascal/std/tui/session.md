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

**Maintenance (implementers only):** keep this file aligned with [`loaded/tui/`](../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) and the standard-unit registry under [`crates/fpas-std/src/std_units/`](../../../../crates/fpas-std/src/std_units/mod.rs).

---

## Importing and names

After `uses Std.Tui;` you can refer to the unit in either form:


| Style               | Example                      |
| ------------------- | ---------------------------- |
| **Fully qualified** | `Std.Tui.Application.Open()` |
| **Short**           | `Application.Open()`         |


`Std.Tui` exports nested names such as `Application.Open`, `Application.Run`, and `EventKind.Resize`. These short forms are available only when `Std.Tui` appears in `uses`.

For the **Rust-hosted dispatch bridge** (`Application.HostProcessNext`, `Application.HostDispatchRedraw`, …), see `[tui-app.md](tui/app.md)`.

`Std.Tui` builds on `[Std.Console](console/README.md)`: the `key` field of `Std.Tui.TuiEvent` has type `**Std.Console.KeyEvent`** (and its `kind` field is `**Std.Console.KeyKind`**). The `**Tui**` prefix avoids clashing with `**Std.Console.Event**`. Import `**Std.Console**` alongside `**Std.Tui**` when you need short names such as `KeyKind` or `WriteLn`, or use fully qualified `Std.Console.*` names.

---

## Current status

`Std.Tui` provides a **hosted dispatch** application path:

- `Application` is the TUI session handle.
- `Size` exposes terminal width and height.
- `TuiEvent` and `EventKind` describe key and resize input for handler signatures.
- `Application.RequestRedraw` marks the session as needing a hosted redraw.
- `Application.Configure` + `Application.Run` register `On*` handlers and run the Rust-hosted loop.

Session lifecycle:

- `Application.Open` starts the terminal session (raw mode and alternate screen when connected to a real terminal).
- `Application.Close` releases that session and restores terminal state.
- `Application.Size` reads the current terminal dimensions.
- `Application.Run` closes the session automatically when the hosted loop exits.

See `[tui-app.md](tui/app.md)` for the full dispatch API, `ApplicationHandlers`, modals, and view-local paint.

---

## Quick reference

Everything below requires `uses Std.Tui;`. Key types for `TuiEvent.key` come from `**Std.Console`** (add `uses Std.Console` for short names like `KeyKind`).


| Kind         | Name                                                                                        | Notes                                                              |
| ------------ | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| type         | `Application`                                                                               | opaque application/session handle                                  |
| type         | `Size`                                                                                      | record with `width` and `height`                                   |
| type         | `EventKind`                                                                                 | enum with `Key` and `Resize`                                       |
| type         | `TuiEvent`                                                                                  | record for one application event (`key` is `Std.Console.KeyEvent`) |
| type         | `Std.Console.KeyKind`                                                                       | enum for logical keys (reused; not defined under `Std.Tui`)        |
| type         | `Std.Console.KeyEvent`                                                                      | record for one key input (reused; not defined under `Std.Tui`)     |
| function     | `Application.Open(): Application`                                                           | create/open an application session                                 |
| procedure    | `Application.Close(App: Application)`                                                       | close the application session                                      |
| function     | `Application.Size(App: Application): Size`                                                  | current terminal size                                              |
| procedure    | `Application.Configure(App: Application; Handlers: ApplicationHandlers)`                    | register hosted `On*` handlers                                     |
| procedure    | `Application.Run(App: Application)`                                                         | run the hosted event loop (closes the session on exit)             |
| procedure    | `Application.RequestRedraw(App: Application)`                                               | mark the application as needing redraw                             |
| enum members | `Std.Console.KeyKind.`* (short `KeyKind.*` with `uses Std.Console`)                         | same as `[Std.Console](console/README.md)`                                |
| enum members | `EventKind.Key`, `EventKind.Resize`                                                         | TUI event kinds                                                    |


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


| Field    | Type      | Meaning                  |
| -------- | --------- | ------------------------ |
| `width`  | `integer` | terminal width in cells  |
| `height` | `integer` | terminal height in cells |


---

### Key input types (`Std.Console.KeyKind`, `Std.Console.KeyEvent`)

`Std.Tui` does not define its own key types. Use `**Std.Console.KeyKind**` and `**Std.Console.KeyEvent**` for `TuiEvent.key` (see `[console.md](console/README.md)` — `ReadKeyEvent`, `KeyEvent`, `KeyKind`). With `uses Std.Console`, the short names `**KeyKind**` and `**KeyEvent**` refer to those console types.

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


| Field  | Type                   | Meaning                          |
| ------ | ---------------------- | -------------------------------- |
| `kind` | `EventKind`            | which event payload is active    |
| `key`  | `Std.Console.KeyEvent` | populated for `EventKind.Key`    |
| `size` | `Size`                 | populated for `EventKind.Resize` |


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

### `procedure Application.RequestRedraw(App: Application)`

Mark the application as needing a redraw. The hosted loop consumes this flag before invoking `OnPaint`.

**Runtime (Rust only):** `TuiSession::is_redraw_pending` in `crates/fpas-std` peeks the same flag without consuming it; used by the VM host when servicing `TuiHostDispatchRedraw` and the bounded `TuiHostRunLoop` path (see `docs/pascal/std/tui/app.md`).

---

## Dispatch model

For full applications, use `Application.Configure` + `Application.Run` instead of a manual event loop. Register `On*` handlers once; the host calls them:

```pascal
program TuiMinimalApplication;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  var S: Size := Application.Size(App);
  ClrScr();
  GotoXY(1, 1);
  WriteLn('Size: ', S.width, 'x', S.height);
  WriteLn('Press Escape to exit')
end;

function OnKeyPressed(App: Application; Key: Std.Console.KeyEvent): boolean;
begin
  if Key.kind = KeyKind.Escape then
  begin
    Application.HostRequestQuit(App);
    return true
  end;
  Application.RequestRedraw(App);
  return false
end;

procedure OnResize(App: Application; NewSize: Size);
begin
  Application.RequestRedraw(App)
end;

begin
  var App: Application := Application.Open();
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
    OnKeyPressed := Some(OnKeyPressed);
    OnResize := Some(OnResize);
  end;
  Application.Configure(App, Handlers);
  Application.Run(App)
end.
```

`Application.Run` performs `Application.Close` automatically after the loop exits. See [Hosted dispatch](app.md) for the full dispatch API and `ApplicationHandlers` fields.

Example: [`examples/pascal/tui/minimal_application.fpas`](../../../../examples/pascal/tui/minimal_application.fpas)

---

## See also

- [Terminal UI index](README.md)
- [Hosted dispatch](app.md)
- [`Std.Console`](../console/README.md)
