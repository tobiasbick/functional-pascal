# `Std.Console`

Text output, line-buffered stdin, and terminal input (classic “CRT” style plus structured key and TUI-ready event APIs). This page is the **authoritative list** of what the unit exports; you do not need to read the implementation to use it.

Add the unit to your program:

```pascal
program Example;
uses Std.Console;
begin
  WriteLn('ok')
end.
```

**Maintenance (implementers only):** keep this file aligned with [`loaded/console.rs`](../../../crates/fpas-sema/src/std_registry/loaded/console.rs), [`console/`](../../../crates/fpas-std/src/console/mod.rs), [`key_event.rs`](../../../crates/fpas-std/src/key_event.rs), [`console_event.rs`](../../../crates/fpas-std/src/console_event.rs), the VM console path in [`vm/`](../../../crates/fpas-vm/src/vm/mod.rs), and [`compiler/std_calls/console.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/console.rs).

---

## Importing and names

After `uses Std.Console;` you can call symbols in either form:

| Style | Example |
|--------|---------|
| **Fully qualified** | `Std.Console.WriteLn('hi')` |
| **Short** | `WriteLn('hi')` |

Short names exist only for symbols that belong to a `uses`’d unit. If two imported units expose the **same** short name (for example `Length` from `Std.Str` and `Std.Array`), the compiler reports an **ambiguous** error at the use site; then use the full name (`Std.Str.Length`, `Std.Array.Length`).

Types follow the same idea: `KeyEvent` is the short form of `Std.Console.KeyEvent` when `Std.Console` is imported.

---

## Quick reference

Everything below requires `uses Std.Console;`.

| Kind | Name | Notes |
|------|------|--------|
| procedure | `Write(...)` | variadic |
| procedure | `WriteLn(...)` | variadic |
| procedure | `ClrScr()` | clear only the active console window |
| procedure | `ClrEol()` | clear from cursor to the right edge of the active window |
| procedure | `GotoXY(X, Y)` | 1-based coordinates inside the active window |
| function | `WhereX(): integer` | current 1-based cursor column inside the active window |
| function | `WhereY(): integer` | current 1-based cursor row inside the active window |
| function | `WindMin(): integer` | packed upper-left corner of the active window (low byte `X`, high byte `Y`) |
| function | `WindMax(): integer` | packed lower-right corner of the active window (low byte `X`, high byte `Y`) |
| procedure | `DelLine()` | delete line at cursor row inside active window |
| procedure | `InsLine()` | insert blank line at cursor row inside active window |
| procedure | `Window(X1, Y1, X2, Y2)` | set the active console window (screen-relative) |
| procedure | `TextColor(Color)` | set foreground color for subsequent writes |
| procedure | `TextBackground(Color)` | set background color for subsequent writes |
| procedure | `TextColorRGB(R, G, B)` | set fg to 24-bit truecolor (0–255 per channel) |
| procedure | `TextBackgroundRGB(R, G, B)` | set bg to 24-bit truecolor (0–255 per channel) |
| procedure | `TextColor256(Index)` | set fg to 256-color palette index (0–255) |
| procedure | `TextBackground256(Index)` | set bg to 256-color palette index (0–255) |
| procedure | `HighVideo()` | set bright foreground intensity bit |
| procedure | `LowVideo()` | clear bright foreground intensity bit |
| procedure | `NormVideo()` | reset attributes to light-gray on black |
| function | `TextAttr(): integer` | packed text attribute (`Background * 16 + Foreground`) |
| procedure | `SetTextAttr(Attr)` | set packed text attribute `0..255` |
| procedure | `Delay(Milliseconds)` | sleep for a non-negative integer number of milliseconds |
| procedure | `CursorOn()` | show the terminal cursor |
| procedure | `CursorOff()` | hide the terminal cursor |
| procedure | `CursorBig()` | show cursor using block style |
| procedure | `TextMode(Mode)` | reset CRT state and store the mode value |
| function | `LastMode(): integer` | last value passed to `TextMode` |
| function | `ScreenWidth(): integer` | current console screen width |
| function | `ScreenHeight(): integer` | current console screen height |
| procedure | `Sound(Hz)` | emit one terminal bell for positive `Hz` |
| procedure | `NoSound()` | stop active tone state (no-op) |
| procedure | `AssignCrt()` | enable CRT mode |
| function | `ReadLn(): string` | line input |
| function | `Read(): char` | same buffer as `ReadLn` |
| function | `ReadKey(): char` | key-by-key, separate from `ReadKeyEvent` |
| function | `KeyPressed(): boolean` | true if `ReadKey` or `ReadKeyEvent` has data waiting |
| function | `ReadKeyEvent(): KeyEvent` | structured key + modifiers |
| function | `EventPending(): boolean` | true if `ReadEvent()` has data waiting |
| function | `ReadEvent(): Event` | unified terminal event for keyboard, mouse, resize, paste, and focus |
| function | `ReadEventTimeout(Milliseconds: integer): Option of Event` | wait up to N ms for an event; requires `EnableRawMode()` first |
| function | `PollEvent(): Option of Event` | non-blocking check; `None` if no event is ready; requires `EnableRawMode()` first |
| procedure | `EnableRawMode()` | explicitly enable terminal raw mode |
| procedure | `DisableRawMode()` | explicitly disable terminal raw mode |
| procedure | `EnterAltScreen()` | switch to the alternate terminal screen |
| procedure | `LeaveAltScreen()` | leave the alternate terminal screen |
| procedure | `EnableMouse()` | enable mouse reporting |
| procedure | `DisableMouse()` | disable mouse reporting |
| procedure | `EnableFocus()` | enable focus gained/lost reporting |
| procedure | `DisableFocus()` | disable focus gained/lost reporting |
| procedure | `EnablePaste()` | enable bracketed paste reporting |
| procedure | `DisablePaste()` | disable bracketed paste reporting |
| type | `KeyEvent` | record |
| type | `KeyKind` | enum |
| type | `Event` | record |
| type | `EventKind` | enum |
| type | `MouseAction` | enum |
| type | `MouseButton` | enum |
| const | `Black`, `Blue`, `Green`, …, `White` | CRT-style color indices `0..15` |
| const | `Blink` | text-attribute blink bit (`128`) |
| const | `BW40`, `C40`, `BW80`, `C80`, `CO40`, `CO80`, `Mono`, `Font8x8` | text-mode compatibility constants |
| enum members | `KeyKind.Unknown`, `KeyKind.Escape`, … | see [KeyKind](#type-keykind-enum) |
| enum members | `EventKind.Key`, `MouseAction.Down`, `MouseButton.Left`, … | see below |

---

## Types

### Type `KeyEvent` (record)

Logical name in the compiler: `Std.Console.KeyEvent`. With `uses Std.Console`, you may write `KeyEvent`.

Equivalent conceptual declaration:

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

| Field | Type | Meaning |
|-------|------|--------|
| `kind` | `KeyKind` | Which key (or `Character` / `Unknown`); see below. |
| `ch` | `char` | For `KeyKind.Character`, the Unicode character; for `KeyKind.Space`, usually `' '`; for most other kinds, often `#0`. |
| `shift` | `boolean` | Shift held when the event was produced. |
| `ctrl` | `boolean` | Control held. |
| `alt` | `boolean` | Alt held. |
| `meta` | `boolean` | Platform “super” / meta where supported. |

**Example:** read one event and branch on the key kind.

```pascal
program Demo;
uses Std.Console;
begin
  var E: KeyEvent := ReadKeyEvent();
  if E.kind = KeyKind.Escape then
    WriteLn('escape')
  else if E.kind = KeyKind.Character then
    WriteLn(E.ch)
  else
    WriteLn('other')
end.
```

You can always use qualified enum literals instead, e.g. `Std.Console.KeyKind.Escape`.

---

### Type `KeyKind` (enum)

Logical name: `Std.Console.KeyKind`. Short: `KeyKind` when `Std.Console` is imported.

Each variant is a **distinct enum value**. You compare with `=` / `<>`, use it in `case` (if your program uses ordinal `case` on enums), and assign to `KeyEvent.kind`.

The language represents the underlying ordinal as an integer index in the **fixed order** below (first row is `0`). You rarely need the number unless you debug; prefer the named variants.

| Index | Variant |
|------:|---------|
| 0 | `Unknown` |
| 1 | `Escape` |
| 2 | `Tab` |
| 3 | `Enter` |
| 4 | `Backspace` |
| 5 | `Space` |
| 6 | `Up` |
| 7 | `Down` |
| 8 | `Left` |
| 9 | `Right` |
| 10 | `Home` |
| 11 | `End` |
| 12 | `PageUp` |
| 13 | `PageDown` |
| 14 | `Insert` |
| 15 | `Delete` |
| 16 | `F1` |
| 17 | `F2` |
| 18 | `F3` |
| 19 | `F4` |
| 20 | `F5` |
| 21 | `F6` |
| 22 | `F7` |
| 23 | `F8` |
| 24 | `F9` |
| 25 | `F10` |
| 26 | `F11` |
| 27 | `F12` |
| 28 | `Character` |

**Literals** (with `uses Std.Console`):

```pascal
var K: KeyKind := KeyKind.Space;
if K = KeyKind.F1 then
  WriteLn('F1');
```

---

### Type `Event` (record)

Logical name: `Std.Console.Event`. Short: `Event` when `Std.Console` is imported.

Equivalent conceptual declaration:

```pascal
type Event = record
  kind: EventKind;
  key: KeyEvent;
  mouse_action: MouseAction;
  mouse_button: MouseButton;
  mouse_x: integer;
  mouse_y: integer;
  width: integer;
  height: integer;
  text: string;
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean
end;
```

`Event` is the low-level event container for later TUI-style code. Only the fields relevant to the current `kind` are populated:

- `Key`: `key` is filled, and the top-level modifier flags mirror `key`.
- `Mouse`: `mouse_action`, `mouse_button`, `mouse_x`, `mouse_y`, and modifiers are filled.
- `Resize`: `width` and `height` are filled.
- `Paste`: `text` is filled.
- `FocusGained` / `FocusLost`: no payload beyond `kind`.

### Type `EventKind` (enum)

Variants in ordinal order:

| Index | Variant |
|------:|---------|
| 0 | `Key` |
| 1 | `Mouse` |
| 2 | `Resize` |
| 3 | `Paste` |
| 4 | `FocusGained` |
| 5 | `FocusLost` |

### Type `MouseAction` (enum)

Variants in ordinal order:

| Index | Variant |
|------:|---------|
| 0 | `Unknown` |
| 1 | `Down` |
| 2 | `Up` |
| 3 | `Drag` |
| 4 | `Move` |
| 5 | `ScrollDown` |
| 6 | `ScrollUp` |
| 7 | `ScrollLeft` |
| 8 | `ScrollRight` |

### Type `MouseButton` (enum)

Variants in ordinal order:

| Index | Variant |
|------:|---------|
| 0 | `None` |
| 1 | `Left` |
| 2 | `Right` |
| 3 | `Middle` |

---

## Procedures

### `procedure Write(...)`

- **Parameters:** zero or more values (variadic). Typical types: `string`, `char`, `integer`, `real`, `boolean`, and other printable runtime values supported by the implementation.
- **Result:** none.
- **Effect:** prints each argument in order **without** appending a newline and **without** inserting separators automatically.

```pascal
Write('count=');
Write(42);
WriteLn('')
```

---

### `procedure WriteLn(...)`

- **Parameters:** zero or more values (same idea as `Write`).
- **Result:** none.
- **Effect:** prints the arguments, then ends the current output line (newline semantics for captures and terminals).

```pascal
WriteLn('Hello, World!');
WriteLn(1, ' ', true);
WriteLn
```

---

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

### `procedure TextColorRGB(R, G, B)`

### `procedure TextBackgroundRGB(R, G, B)`

- **Parameters:** three integers `R`, `G`, `B` (0–255 each).
- **Result:** none.
- **Effect:** applies a 24-bit truecolor ANSI escape for the foreground / background. Takes effect immediately for subsequent `Write`/`WriteLn` calls.
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
- **Errors:** runtime error if the index is outside `0..255`.

```pascal
uses Std.Console;

TextColor256(196);        { bright red in xterm-256color }
WriteLn('256-color text');
```

### `function TextAttr(): integer`

- **Parameters:** none.
- **Returns:** packed text attribute as `Background * 16 + Foreground`.

### `procedure SetTextAttr(Attr)`

- **Parameters:** `Attr: integer`.
- **Result:** none.
- **Accepted values:** `0..255`.
- **Effect:** unpacks and applies foreground/background colors from the packed attribute.
- **Errors:** runtime error if `Attr` is outside `0..255`.

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

---

## Functions (text input)

These share one **line-oriented** buffer: typed text and test “stdin” lines are consumed in order.

### `function ReadLn(): string`

- **Parameters:** none.
- **Returns:** the next full line, **without** the line terminator.
- **Buffer:** same stream as `Read()`.

```pascal
var Line: string := ReadLn();
WriteLn(Line)
```

---

### `function Read(): char`

- **Parameters:** none.
- **Returns:** the next single character from the **current** line buffer (or the next line’s data as exposed by the runtime).
- **Buffer:** same as `ReadLn()`.

```pascal
var C: char := Read();
WriteLn(C)
```

---

## Functions (keyboard)

Keyboard input is **separate** from the `Read` / `ReadLn` buffer. Enabling raw or low-level keyboard mode is handled by the runtime when you call these.

### `function ReadKey(): char`

- **Parameters:** none.
- **Returns:** one character from the keyboard queue.
- **Notes:** does not wait for Enter. **Extended keys** (arrows, function keys, etc.) may appear as a **two-step** sequence: first `#0`, then a second `char` encoding the physical key (Turbo Pascal–style).

```pascal
var C: char := ReadKey();
WriteLn(C)
```

---

### `function KeyPressed(): boolean`

- **Parameters:** none.
- **Returns:** `true` if a value is ready for **either** `ReadKey()` **or** `ReadKeyEvent()` (whichever queue has data), else `false`.

Use it to avoid blocking when you want a polling loop.

```pascal
if KeyPressed() then
begin
  var C: char := ReadKey();
  WriteLn(C)
end
```

---

### `function ReadKeyEvent(): KeyEvent`

- **Parameters:** none.
- **Returns:** one `KeyEvent` with `kind`, `ch`, and modifier flags.

**Queues:** `ReadKey()` and `ReadKeyEvent()` use **different** internal queues. Characters you inject for `ReadKey` tests do **not** show up in `ReadKeyEvent`, and structured events queued for `ReadKeyEvent` do **not** satisfy `ReadKey`.

**Mapping (typical console):**

- **Space bar:** `kind = KeyKind.Space`, `ch` is often `' '`.
- **Printable keys:** `kind = KeyKind.Character`, `ch` is the character.
- **Special keys:** dedicated `KeyKind` values (`Enter`, arrows, `F1`…`F12`, etc.); `ch` is often `#0`.
- **Unknown / unmapped:** `kind = KeyKind.Unknown`.

```pascal
if KeyPressed() then
begin
  var E: KeyEvent := ReadKeyEvent();
  if E.kind = KeyKind.Escape then
    WriteLn('quit')
  else
    WriteLn(E.ch)
end
```

### `function EventPending(): boolean`

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

---

## Using text and keyboard together

- Use **`Read` / `ReadLn`** for typed input and pipes (line discipline).
- Use **`ReadKey` / `ReadKeyEvent`** for games or immediate key handling.
- Do not assume that mixing the two in one tight loop will interleave predictably without designing your loop; they are different subsystems.

---

## Implementation map (for contributors)

| Concern | Location |
|---------|-----------|
| User-facing registration | [`loaded/console.rs`](../../../crates/fpas-sema/src/std_registry/loaded/console.rs) |
| Console backend | [`console/mod.rs`](../../../crates/fpas-std/src/console/mod.rs) |
| Key and event types | [`key_event.rs`](../../../crates/fpas-std/src/key_event.rs), [`console_event.rs`](../../../crates/fpas-std/src/console_event.rs) |
| Bytecode / VM | [`intrinsic/mod.rs`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs), [`vm/mod.rs`](../../../crates/fpas-vm/src/vm/mod.rs) |
| Code generation | [`std_calls/console.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/console.rs) |

[← Standard library index](README.md)
