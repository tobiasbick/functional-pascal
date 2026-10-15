# Case and trim

## `Text.Length(): integer`

Returns how many characters are in `S` (scalar count).

```pascal id=example-1 fragment context=body-console
const N: integer := 'café'.Length();
Console.WriteLn(N);
```

---

## `Text.ToUpper(): string`

Returns a new string with letters uppercased (Unicode-aware where the runtime supports it).

```pascal id=example-2 fragment context=body-console
Console.WriteLn('ab'.ToUpper());
```

---

## `Text.ToLower(): string`

Returns a new string with letters lowercased.

```pascal id=example-3 fragment context=body-console
Console.WriteLn('AB'.ToLower());
```

---

## `Text.Trim(): string`

Strips leading and trailing whitespace.

```pascal id=example-4 fragment context=body-console
Console.WriteLn('  x  '.Trim());
```

---

## `Text.TrimLeft(): string`

Strips leading whitespace only.

```pascal id=example-5 fragment context=body-console
Console.WriteLn('  hi  '.TrimLeft()); // 'hi  '
```

---

## `Text.TrimRight(): string`

Strips trailing whitespace only.

```pascal id=example-6 fragment context=body-console
Console.WriteLn('  hi  '.TrimRight()); // '  hi'
```

## See also

- [Str overview](README.md)
- [Search](search.md)
- [Text index](../../../std/text/README.md)
