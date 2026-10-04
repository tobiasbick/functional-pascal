# Case and trim

## `function Length(S: string): integer`

Returns how many characters are in `S` (scalar count).

```pascal
uses Std.Console as Console;

const N: integer := Length('café');
Console.WriteLn(N);
```

---

## `function ToUpper(S: string): string`

Returns a new string with letters uppercased (Unicode-aware where the runtime supports it).

```pascal
uses Std.Console as Console;

Console.WriteLn(ToUpper('ab'));
```

---

## `function ToLower(S: string): string`

Returns a new string with letters lowercased.

```pascal
uses Std.Console as Console;

Console.WriteLn(ToLower('AB'));
```

---

## `function Trim(S: string): string`

Strips leading and trailing whitespace.

```pascal
uses Std.Console as Console;

Console.WriteLn(Trim('  x  '));
```

---

## `function TrimLeft(S: string): string`

Strips leading whitespace only.

```pascal
uses Std.Console as Console;

Console.WriteLn(TrimLeft('  hi  ')); // 'hi  '
```

---

## `function TrimRight(S: string): string`

Strips trailing whitespace only.

```pascal
uses Std.Console as Console;

Console.WriteLn(TrimRight('  hi  ')); // '  hi'
```

## See also

- [Str overview](README.md)
- [Search](search.md)
- [Text index](../README.md)
