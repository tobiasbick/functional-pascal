# Colors and attributes

Extended color procedures (`TextColorRGB`, `TextBackgroundRGB`, `TextColor256`, `TextBackground256`) send terminal ANSI color escapes directly. They do not update the packed 16-color CRT attribute returned by `TextAttr()`. Calling `TextColor`, `TextBackground`, `HighVideo`, `LowVideo`, `NormVideo`, or `SetTextAttr` afterwards switches back to the packed CRT attribute path and overrides the extended color.

---

---

### `procedure TextColor(Color)`

### `procedure TextBackground(Color)`

- **Parameters:** one integer color index.
- **Result:** none.
- **Accepted values:** `0..15`.
- **Effect:** changes the foreground/background color used for subsequent writes.
- **Errors:** runtime error if the color index is outside `0..15`.

The following CRT-style constants are available after `uses Std.Console;`:

| Value | Constant |
|------:|----------|
| 0 | `Black` |
| 1 | `Blue` |
| 2 | `Green` |
| 3 | `Cyan` |
| 4 | `Red` |
| 5 | `Magenta` |
| 6 | `Brown` |
| 7 | `LightGray` |
| 8 | `DarkGray` |
| 9 | `LightBlue` |
| 10 | `LightGreen` |
| 11 | `LightCyan` |
| 12 | `LightRed` |
| 13 | `LightMagenta` |
| 14 | `Yellow` |
| 15 | `White` |

Additional CRT compatibility constants:

| Value | Constant |
|------:|----------|
| 128 | `Blink` |
| 0 | `BW40` |
| 1 | `C40` |
| 2 | `BW80` |
| 3 | `C80` |
| 4 | `CO40` |
| 5 | `CO80` |
| 7 | `Mono` |
| 256 | `Font8x8` |

### `procedure HighVideo()`

### `procedure LowVideo()`

### `procedure NormVideo()`

- **Parameters:** none.
- **Result:** none.
- **Effect (`HighVideo`):** sets the foreground intensity bit.
- **Effect (`LowVideo`):** clears the foreground intensity bit.
- **Effect (`NormVideo`):** resets attributes to light gray on black (`TextAttr = 7`).
- **Interaction:** `NormVideo` replaces any active RGB / 256-color styling with the normal packed CRT attribute state.

### `procedure TextColorRGB(R, G, B)`

### `procedure TextBackgroundRGB(R, G, B)`

- **Parameters:** three integers `R`, `G`, `B` (0–255 each).
- **Result:** none.
- **Effect:** applies a 24-bit truecolor ANSI escape for the foreground / background. Takes effect immediately for subsequent `Write`/`WriteLn` calls.
- **State interaction:** does not change the packed value returned by `TextAttr()`. A later call to `TextColor`, `TextBackground`, `HighVideo`, `LowVideo`, `NormVideo`, or `SetTextAttr` overrides the RGB color.
- **Errors:** runtime error if any channel is outside `0..255`.

```pascal
uses Std.Console;

TextColorRGB(255, 128, 0);       { orange foreground }
TextBackgroundRGB(0, 0, 64);     { dark-blue background }
WriteLn('truecolor text');
```

### `procedure TextColor256(Index)`

### `procedure TextBackground256(Index)`

- **Parameters:** one integer index (0–255).
- **Result:** none.
- **Effect:** applies a 256-color ANSI palette escape for the foreground / background. Takes effect immediately for subsequent `Write`/`WriteLn` calls.
- **State interaction:** does not change the packed value returned by `TextAttr()`. A later call to `TextColor`, `TextBackground`, `HighVideo`, `LowVideo`, `NormVideo`, or `SetTextAttr` overrides the 256-color selection.
- **Errors:** runtime error if the index is outside `0..255`.

```pascal
uses Std.Console;

TextColor256(196);        { bright red in xterm-256color }
WriteLn('256-color text');
```

### `function TextAttr(): integer`

- **Parameters:** none.
- **Returns:** packed text attribute as `Background * 16 + Foreground`.
- **Scope:** reports only the packed CRT 16-color attribute state. Extended colors set by `TextColorRGB`, `TextBackgroundRGB`, `TextColor256`, and `TextBackground256` are not representable in this integer and therefore do not change the returned value.

### `procedure SetTextAttr(Attr)`

- **Parameters:** `Attr: integer`.
- **Result:** none.
- **Accepted values:** `0..255`.
- **Effect:** unpacks and applies foreground/background colors from the packed attribute.
- **Interaction:** overrides any active RGB / 256-color styling and returns output to the packed CRT 16-color attribute model.
- **Errors:** runtime error if `Attr` is outside `0..255`.

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
