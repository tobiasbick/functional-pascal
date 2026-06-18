# Terminal events

- **Parameters:** none.
- **Returns:** `true` if a unified terminal event is queued for `ReadEvent()`.

### `function ReadEvent(): Event`

- **Parameters:** none.
- **Returns:** one `Event`.
- **Events:** `Key`, `Mouse`, `Resize`, `Paste`, `FocusGained`, `FocusLost`.

`ReadEvent()` is intended as the low-level terminal event API for TUI-style code. It uses a queue that is separate from the classic `ReadKey()` / `ReadKeyEvent()` APIs.

```pascal
var E: Event := ReadEvent();
if E.kind = EventKind.Resize then
  WriteLn(E.width, 'x', E.height);
```

### `function ReadEventTimeout(Milliseconds: integer): Option of Event`

- **Parameters:** `Milliseconds` — maximum time to wait in milliseconds (`0` = non-blocking poll).
- **Returns:** `Some(E)` if an event arrived within the timeout; `None` otherwise.
- **Prerequisite:** call `EnableRawMode()` before using this function. If raw mode is not active, `None` is returned immediately.

```pascal
uses Std.Console, Std.Option;

EnableRawMode();
var MaybeEvent: Option of Event := ReadEventTimeout(100);
match MaybeEvent with
  | Some(E) => WriteLn(E.kind)
  | None => WriteLn('timeout')
end
```

### `function PollEvent(): Option of Event`

- **Parameters:** none.
- **Returns:** `Some(E)` if an event is already available; `None` if the queue is empty.
- **Prerequisite:** call `EnableRawMode()` before using this function. If raw mode is not active, `None` is returned immediately.

```pascal
uses Std.Console, Std.Option;

EnableRawMode();
var MaybeE: Option of Event := PollEvent();
match MaybeE with
  | Some(E) => WriteLn('got event')
  | None => WriteLn('nothing pending')
end
```

### `procedure DisableRawMode()`

### `procedure EnterAltScreen()`

### `procedure LeaveAltScreen()`

### `procedure EnableMouse()`

### `procedure DisableMouse()`

### `procedure EnableFocus()`

### `procedure DisableFocus()`

### `procedure EnablePaste()`

### `procedure DisablePaste()`

- **Parameters:** none.
- **Result:** none.
- **Effect:** toggles the corresponding terminal feature when the runtime is attached to an interactive terminal.

## See also

- [Console overview](README.md)
- [Types](types.md)
- [Using together](using-together.md)
