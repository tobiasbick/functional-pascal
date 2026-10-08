# Split and join

## `Text.Split(Delim: string): array of string`

Splits `S` around each occurrence of `Delim`. Returns a new array of segments.

- **`Delim` must not be empty** — empty delimiter is a **runtime error**.

```pascal
program SplitDemo;
uses Std.Console;
begin
  const Parts: array of string := 'x,y'.Split(',');
  WriteLn(Parts.Length());
end.
```

`Parts.Length()` selects the array operation by the receiver's type even with `String operations` also imported.

---

## `function Join(Parts: array of string; Delim: string): string`

Concatenates every element of `Parts`, inserting `Delim` between elements.

```pascal
WriteLn(['x', 'y'].Join(':'));
```

---

## See also

- [Str overview](README.md)
- [Edit](edit.md)
- [Text index](../../../std/text/README.md)
