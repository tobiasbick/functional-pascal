# Search

## `Text.Contains(Sub: string): boolean`

`true` if `Sub` occurs anywhere in `S`, else `false`.

```pascal
if 'abc'.Contains('b') then
  Console.WriteLn('yes');
end if;
```

---

## `Text.StartsWith(Pre: string): boolean`

`true` if `S` begins with `Pre`.

```pascal
Console.WriteLn('abc'.StartsWith('ab'));
```

---

## `Text.EndsWith(Suf: string): boolean`

`true` if `S` ends with `Suf`.

```pascal
Console.WriteLn('abc'.EndsWith('bc'));
```

---

## `Text.Slice(Start: integer; Len: integer): string`

Copies `Len` characters starting at `Start`. **Bounds are checked at runtime**; invalid ranges produce a runtime error.

```pascal
Console.WriteLn('Hello'.Slice(0, 3));
```

---

## `Text.IndexOf(Sub: string): integer`

Returns the **first** character index of `Sub` in `S`, or **`-1`** if not found.

```pascal
Console.WriteLn('aba'.IndexOf('a'));
Console.WriteLn('aba'.IndexOf('z'));
```

---

## `Text.LastIndexOf(Sub: string): integer`

Returns the **last** character index of `Sub` in `S`, or **`-1`** if not found.

```pascal
Console.WriteLn('abcabc'.LastIndexOf('abc'));  // 3
Console.WriteLn('abc'.LastIndexOf('z'));       // -1
```

## See also

- [Str overview](README.md)
- [Split and join](split-join.md)
- [Text index](../../../std/text/README.md)
