# Split and join

## `function Split(S: string; Delim: string): array of (string)`

Splits `S` around each occurrence of `Delim`. Returns a new array of segments.

- **`Delim` must not be empty** — empty delimiter is a **runtime error**.

```pascal
program SplitDemo;

uses Std.Console as Console;
uses Std.Str as Str;
uses Std.Arrays as Arrays;

begin
  const Parts: array of (string) := Str.Split('x,y', ',');
  Console.WriteLn(Arrays.Length(Parts));
end program;
```

(`Length` for arrays would be ambiguous with `Std.Str` also imported; qualify `Std.Arrays.Length` here.)

---

## `function Join(Parts: array of (string); Delim: string): string`

Concatenates every element of `Parts`, inserting `Delim` between elements.

```pascal
uses Std.Console as Console;

Console.WriteLn(Join(['x', 'y'], ':'));
```

---

## See also

- [Str overview](README.md)
- [Edit](edit.md)
- [Text index](../README.md)
