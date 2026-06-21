# Types

## Types

### Type `KeyEvent` (record)

Logical name in the compiler: `Std.Console.KeyEvent`. With `uses Std.Console`, you may write `KeyEvent`.

Equivalent conceptual declaration:

```pascal
type KeyEvent = record
  kind: KeyKind;
  ch: string;
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean
end;
```

| Field | Type | Meaning |
|-------|------|--------|
| `kind` | `KeyKind` | Which key (or `Character` / `Unknown`); see below. |
| `ch` | `string` | For `KeyKind.Character`, the Unicode character; for `KeyKind.Space`, usually `' '`; for most other kinds, often `''`. |
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

## See also

- [Console overview](README.md)
- [Keyboard](keyboard.md)
- [Terminal events](events.md)
