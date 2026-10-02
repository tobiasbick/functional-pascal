# Output

## Procedures

### `procedure WriteText(...)`

- **Parameters:** zero or more values (variadic). Typical types: `string`, `string`, `integer`, `real`, `boolean`, and other printable runtime values supported by the implementation.
- **Result:** none.
- **Effect:** prints each argument in order **without** appending a newline and **without** inserting separators automatically.

```pascal
uses Std.Console as Console;

Console.WriteText('count=');
Console.WriteText(42);
Console.WriteLn('');
```

---

### `procedure WriteLn(...)`

- **Parameters:** zero or more values (same idea as `WriteText`).
- **Result:** none.
- **Effect:** prints the arguments, then ends the current output line (newline semantics for captures and terminals).

```pascal
uses Std.Console as Console;

Console.WriteLn('Hello, World!');
Console.WriteLn(1, ' ', true);
Console.WriteLn();
```

---

## See also

- [Console overview](README.md)
- [Screen control](screen.md)
- [Colors and attributes](colors.md)
