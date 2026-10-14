# Case and trim

## `Text.Length(): integer`

Returns how many characters are in `S` (scalar count).

```pascal
const N: integer := 'café'.Length();
Console.WriteLn(N);
```

---

## `Text.ToUpper(): string`

Returns a new string with letters uppercased (Unicode-aware where the runtime supports it).

```pascal
Console.WriteLn('ab'.ToUpper());
```

---

## `Text.ToLower(): string`

Returns a new string with letters lowercased.

```pascal
Console.WriteLn('AB'.ToLower());
```

---

## `Text.Trim(): string`

Strips leading and trailing whitespace.

```pascal
Console.WriteLn('  x  '.Trim());
```

---

## `Text.TrimLeft(): string`

Strips leading whitespace only.

```pascal
Console.WriteLn('  hi  '.TrimLeft()); // 'hi  '
```

---

## `Text.TrimRight(): string`

Strips trailing whitespace only.

```pascal
Console.WriteLn('  hi  '.TrimRight()); // '  hi'
```

## See also

- [Str overview](README.md)
- [Search](search.md)
- [Text index](../../../std/text/README.md)
