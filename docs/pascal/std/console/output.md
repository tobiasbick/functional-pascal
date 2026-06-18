# Output

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

## See also

- [Console overview](README.md)
- [Screen control](screen.md)
- [Colors and attributes](colors.md)
