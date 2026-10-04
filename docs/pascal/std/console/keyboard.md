# Keyboard

## Functions (keyboard)

Keyboard input is **separate** from the `ReadText` / `ReadLn` buffer. Enabling raw or low-level keyboard mode is handled by the runtime when you call these.

### `function ReadKey(): string`

- **Parameters:** none.
- **Returns:** one character from the keyboard queue.
- **Notes:** does not wait for Enter. **Extended keys** (arrows, function keys, etc.) may appear as a **two-step** sequence: first `''`, then a second `string` encoding the physical key (Turbo Pascal–style).

```pascal
uses Std.Console as Console;

const C: string := Console.ReadKey();
Console.WriteLn(C);
```

---

### `function KeyPressed(): boolean`

- **Parameters:** none.
- **Returns:** `true` if a value is ready for **either** `ReadKey()` **or** `ReadKeyEvent()` (whichever queue has data), else `false`.

Use it to avoid blocking when you want a polling loop.

```pascal
uses Std.Console as Console;

if Console.KeyPressed() then
  begin
    const C: string := Console.ReadKey();
    Console.WriteLn(C);
  end;
end if;
```

---

### `function ReadKeyEvent(): KeyEvent`

- **Parameters:** none.
- **Returns:** one `KeyEvent` with `kind`, `ch`, and modifier flags.

**Queues:** `ReadKey()` and `ReadKeyEvent()` use **different** internal queues. Characters you inject for `ReadKey` tests do **not** show up in `ReadKeyEvent`, and structured events queued for `ReadKeyEvent` do **not** satisfy `ReadKey`.

**Mapping (typical console):**

- **Space bar:** `kind = KeyKind.Space`, `ch` is often `' '`.
- **Printable keys:** `kind = KeyKind.Character`, `ch` is the character.
- **Special keys:** dedicated `KeyKind` values (`Enter`, arrows, `F1`…`F12`, etc.); `ch` is often `''`.
- **Unknown / unmapped:** `kind = KeyKind.Unknown`.

```pascal
uses Std.Console as Console;

if Console.KeyPressed() then
  begin
    const E: Console.KeyEvent := Console.ReadKeyEvent();
    if E.kind = Console.KeyKind.Escape then
      Console.WriteLn('quit');
    else
      Console.WriteLn(E.ch);
    end if;
  end;
end if;
```

### `function EventPending(): boolean`

- **Parameters:** none.
- **Returns:** `true` if a unified terminal event is queued for `ReadEvent()`, otherwise `false`.
- **Notes:** observes the unified event queue used by `ReadEvent`, `ReadEventTimeout`, and `PollEvent`.

## See also

- [Console overview](README.md)
- [Types](types.md)
- [Using together](using-together.md)
