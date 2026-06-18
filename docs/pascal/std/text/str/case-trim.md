# Case and trim

## `function Length(S: string): integer`

Returns how many characters are in `S` (scalar count).

```pascal
var N: integer := Length('café');
WriteLn(N)
```

---

## `function ToUpper(S: string): string`

Returns a new string with letters uppercased (Unicode-aware where the runtime supports it).

```pascal
WriteLn(ToUpper('ab'))
```

---

## `function ToLower(S: string): string`

Returns a new string with letters lowercased.

```pascal
WriteLn(ToLower('AB'))
```

---

## `function Trim(S: string): string`

Strips leading and trailing whitespace.

```pascal
WriteLn(Trim('  x  '))
```

---

## `function TrimLeft(S: string): string`

Strips leading whitespace only.

```pascal
WriteLn(TrimLeft('  hi  '))  { 'hi  ' }
```

---

## `function TrimRight(S: string): string`

Strips trailing whitespace only.

```pascal
WriteLn(TrimRight('  hi  '))  { '  hi' }
```

## See also

- [Str overview](README.md)
- [Search](search.md)
- [Text index](../README.md)
