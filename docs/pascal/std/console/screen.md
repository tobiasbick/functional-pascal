# Screen control

## Procedures (screen control)

These operations use a **text screen model** with a current cursor position, an active window, and CRT-style colors.

### `procedure ClrScr()`

- **Parameters:** none.
- **Result:** none.
- **Effect:** clears only the **active window** and moves the cursor to `(1, 1)` inside that window.

### `procedure ClrEol()`

- **Parameters:** none.
- **Result:** none.
- **Effect:** clears from the current cursor position to the right edge of the active window.

### `procedure GotoXY(X, Y)`

- **Parameters:** `X: integer`, `Y: integer`.
- **Result:** none.
- **Coordinates:** **1-based** and **relative to the active window**.
- **Errors:** runtime error if `X` / `Y` are outside the active window.

```pascal
Window(10, 5, 30, 15);
GotoXY(1, 1);
WriteLn('top-left of the window')
```

### `function WhereX(): integer`

- **Parameters:** none.
- **Returns:** current **1-based** cursor column inside the active window.

### `function WhereY(): integer`

- **Parameters:** none.
- **Returns:** current **1-based** cursor row inside the active window.

### `function WindMin(): integer`

- **Parameters:** none.
- **Returns:** packed upper-left corner of the active window.
- **Packing:** low byte is `X`, high byte is `Y` (CRT-style word packing).

### `function WindMax(): integer`

- **Parameters:** none.
- **Returns:** packed lower-right corner of the active window.
- **Packing:** low byte is `X`, high byte is `Y` (CRT-style word packing).

### `procedure DelLine()`

- **Parameters:** none.
- **Result:** none.
- **Effect:** removes the line at the current cursor row inside the active window, shifts lower lines up, and clears the bottom line.

### `procedure InsLine()`

- **Parameters:** none.
- **Result:** none.
- **Effect:** inserts a blank line at the current cursor row inside the active window, shifts lower lines down, and drops the bottom line.

### `procedure Window(X1, Y1, X2, Y2)`

- **Parameters:** four integers.
- **Result:** none.
- **Coordinates:** **1-based**, **inclusive**, and **screen-relative**.
- **Effect:** sets the active text window; after the call the cursor becomes `(1, 1)` inside that window.
- **Errors:** runtime error if the rectangle is outside the current screen or if `X1 > X2` / `Y1 > Y2`.

`Window` is a **text viewport**, not a GUI window. `GotoXY`, `WhereX`, `WhereY`, `ClrScr`, `ClrEol`, and scrolling are all relative to this rectangle.

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
