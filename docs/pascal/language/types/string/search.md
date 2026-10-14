# Search

## Membership with `in`

`Sub in Text` is `true` if Sub occurs anywhere in Text, else `false`.
An empty substring belongs to every string, including the empty string.

```pascal
if 'b' in 'abc' then
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

## `Text.IndexOf(Sub: string): Option of integer`

Returns `Some(Index)` for the first occurrence of Sub in Text, or `None` if
not found. Indexes count zero-based Unicode scalars, rather than UTF-8 bytes or
grapheme clusters. An empty substring returns `Some(0)`, even for an empty Text.

```pascal
case 'aé😀'.IndexOf('😀') of
  when Some(const Index):
    Console.WriteLn(Index); // 2
  when None:
    Console.WriteLn('absent');
end case;

Console.WriteLn('aba'.IndexOf('z')); // None
```

---

## `Text.LastIndexOf(Sub: string): Option of integer`

Returns `Some(Index)` for the last occurrence of Sub in Text, or `None` if
not found. Indexes count Unicode scalars. An empty substring returns
`Some(Text.Length())`; an empty receiver and empty substring return `Some(0)`.

```pascal
Console.WriteLn('abcabc'.LastIndexOf('abc')); // Some(3)
Console.WriteLn('abc'.LastIndexOf('z')); // None
```

## See also

- [Str overview](README.md)
- [Split and join](split-join.md)
- [Text index](../../../std/text/README.md)
