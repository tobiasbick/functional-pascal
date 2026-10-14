# Split and join

## `Text.Split(Delim: string): array of string`

Splits `S` around each occurrence of `Delim`. Returns a new array of segments.

- **`Delim` must not be empty** — empty delimiter is a **runtime error**.

```pascal
program SplitDemo;

uses Std.Console as Console;

begin
  const Parts: array of string := 'x,y'.Split(',');
  Console.WriteLn(Parts.Length());
end.
```

`Parts.Length()` selects the native array operation by the receiver's type.
Native string and array operations require no imports.

---

## `Parts.Join(Delim: string): string`

Concatenates every element of `Parts`, inserting `Delim` between elements.

```pascal
Console.WriteLn(['x', 'y'].Join(':'));
```

---

## See also

- [Str overview](README.md)
- [Edit](edit.md)
- [Text index](../../../std/text/README.md)
