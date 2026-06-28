# Screen utilities

### `procedure Delay(Milliseconds)`

- **Parameters:** `Milliseconds: integer`.
- **Result:** none.
- **Effect:** pauses execution for the requested time.
- **Errors:** runtime error if the value is negative.

### `procedure CursorOn()`

### `procedure CursorOff()`

### `procedure CursorBig()`

- **Parameters:** none.
- **Result:** none.
- **Effect (`CursorOn`):** shows the terminal cursor.
- **Effect (`CursorOff`):** hides the terminal cursor.
- **Effect (`CursorBig`):** shows the cursor with block style.

### `procedure TextMode(Mode)`

- **Parameters:** `Mode: integer`.
- **Result:** none.
- **Effect:** stores `Mode` in `LastMode`, resets attributes/cursor, clears the screen, and restores the full-screen active window.
- **Errors:** runtime error if `Mode` is negative.

### `function LastMode(): integer`

- **Parameters:** none.
- **Returns:** last value passed to `TextMode`.

### `function ScreenWidth(): integer`

### `function ScreenHeight(): integer`

- **Parameters:** none.
- **Returns:** current console screen width/height in character cells.

### `procedure Sound(Hz)`

- **Parameters:** `Hz: integer`.
- **Result:** none.
- **Effect:** emits one terminal bell when `Hz > 0`.
- **Errors:** runtime error if `Hz <= 0`.

### `procedure NoSound()`

- **Parameters:** none.
- **Result:** none.
- **Effect:** ends active tone state.

### `procedure AssignCrt()`

- **Parameters:** none.
- **Result:** none.
- **Effect:** enables CRT-mode rendering.

## Example

```pascal
uses Std.Console;

begin
  AssignCrt();
  TextMode(C80);
  CursorOff();
  Delay(100);
  CursorOn()
end.
```

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
