# Terminal events

- **Parameters:** none.
- **Returns:** `true` if a unified terminal event is queued for `ReadEvent()`.

### `function ReadEvent(): Event`

- **Parameters:** none.
- **Returns:** one `Event`.
- **Events:** `Key`, `Mouse`, `Resize`, `Paste`, `FocusGained`, `FocusLost`.

`ReadEvent()` is intended as the low-level terminal event API for TUI-style code. Test injection uses a queue that is separate from the classic `ReadKey()` / `ReadKeyEvent()` APIs.

Live terminal keys are mirrored so both `KeyPressed` / `EventPending` can observe them, but **consuming a live key from either API removes it from both**. Prefer one live input style per loop (`ReadEvent*` **or** `ReadKey*`), not both.

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

### `procedure AcquireInteractiveTerminal()`

- **Parameters:** none.
- **Effect:** acquires exclusive interactive terminal ownership. When a terminal writer is attached,
  enables raw mode (on a real TTY), the alternate screen, mouse, focus, and paste reporting, then
  hides the cursor. Failed steps roll back earlier steps. Without a writer the call only records
  ownership so a second acquire fails.
- **Errors:** a second acquire while one session is already open is a runtime error.

### `procedure ReleaseInteractiveTerminal()`

- **Parameters:** none.
- **Effect:** restores modes owned by the matching acquire, in reverse order. Idempotent when
  nothing is acquired. Console teardown also restores owned screen modes as a safety net; raw mode
  remains restored by `KeyInput` teardown when it was enabled.

## See also

- [Console overview](README.md)
- [Types](types.md)
- [Using together](using-together.md)
