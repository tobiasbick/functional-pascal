# Line input

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

### `function Read(): string`

- **Parameters:** none.
- **Returns:** the next single character from the **current** line buffer (or the next line’s data as exposed by the runtime).
- **Buffer:** same as `ReadLn()`.

```pascal
var C: string := Read();
WriteLn(C)
```

---

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
