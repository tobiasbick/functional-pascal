# Case and trim

## `Text.Length(): integer`

Returns how many characters are in `S` (scalar count).

```pascal
const N: integer := 'café'.Length();
WriteLn(N);
```

---

## `Text.ToUpper(): string`

Returns a new string with letters uppercased (Unicode-aware where the runtime supports it).

```pascal
WriteLn('ab'.ToUpper());
```

---

## `Text.ToLower(): string`

Returns a new string with letters lowercased.

```pascal
WriteLn('AB'.ToLower());
```

---

## `Text.Trim(): string`

Strips leading and trailing whitespace.

```pascal
WriteLn('  x  '.Trim());
```

---

## `Text.TrimLeft(): string`

Strips leading whitespace only.

```pascal
WriteLn('  hi  '.TrimLeft());  // 'hi  '
```

---

## `Text.TrimRight(): string`

Strips trailing whitespace only.

```pascal
WriteLn('  hi  '.TrimRight());  // '  hi'
```

## See also

- [Str overview](README.md)
- [Search](search.md)
- [Text index](../../../std/text/README.md)
