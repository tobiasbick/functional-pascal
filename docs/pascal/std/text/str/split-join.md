# Split and join

## `function Split(S: string; Delim: string): array of string`

Splits `S` around each occurrence of `Delim`. Returns a new array of segments.

- **`Delim` must not be empty** — empty delimiter is a **runtime error**.

```pascal
program SplitDemo;
uses Std.Console, Std.Str, Std.Arrays;
begin
  var Parts: array of string := Split('x,y', ',');
  WriteLn(Std.Arrays.Length(Parts))
end.
```

(`Length` for arrays would be ambiguous with `Std.Str` also imported; qualify `Std.Arrays.Length` here.)

---

## `function Join(Parts: array of string; Delim: string): string`

Concatenates every element of `Parts`, inserting `Delim` between elements.

```pascal
WriteLn(Join(['x', 'y'], ':'))
```

---

## See also

- [Str overview](README.md)
- [Edit](edit.md)
- [Text index](../README.md)
