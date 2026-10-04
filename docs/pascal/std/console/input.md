# Line input

## Functions (text input)

These share one **line-oriented** buffer: typed text and test “stdin” lines are consumed in order.

### `function ReadLn(): string`

- **Parameters:** none.
- **Returns:** the next full line, **without** the line terminator.
- **Buffer:** same stream as `ReadText()`.

```pascal
uses Std.Console as Console;

const Line: string := Console.ReadLn();
Console.WriteLn(Line);
```

---

### `function ReadText(): string`

- **Parameters:** none.
- **Returns:** the next single character from the **current** line buffer (or the next line’s data as exposed by the runtime).
- **Buffer:** same as `ReadLn()`.

```pascal
uses Std.Console as Console;

const C: string := Console.ReadText();
Console.WriteLn(C);
```

---

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
