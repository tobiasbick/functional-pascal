# Search

## Membership with `in`

`Sub in Text` is `true` if Sub occurs anywhere in Text, else `false`.
An empty substring belongs to every string, including the empty string.

```pascal id=example-1 fragment context=body-console
if 'b' in 'abc' then
  Console.WriteLn('yes');
end if;
```

---

## `Text.StartsWith(Pre: string): boolean`

`true` if `S` begins with `Pre`.

```pascal id=example-2 fragment context=body-console
Console.WriteLn('abc'.StartsWith('ab'));
```

---

## `Text.EndsWith(Suf: string): boolean`

`true` if `S` ends with `Suf`.

```pascal id=example-3 fragment context=body-console
Console.WriteLn('abc'.EndsWith('bc'));
```

---

## `Text.Slice(Start: integer; Len: integer): string`

Copies `Len` characters starting at `Start`. **Bounds are checked at runtime**; invalid ranges produce a runtime error.

```pascal id=example-4 fragment context=body-console
Console.WriteLn('Hello'.Slice(0, 3));
```

---

## `Text.IndexOf(Sub: string): Option of integer`

Returns `Some(Index)` for the first occurrence of Sub in Text, or `None` if
not found. Indexes count zero-based Unicode scalars, rather than UTF-8 bytes or
grapheme clusters. An empty substring returns `Some(0)`, even for an empty Text.

```pascal id=example-5 fragment context=body-console
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

```pascal id=example-6 fragment context=body-console
Console.WriteLn('abcabc'.LastIndexOf('abc')); // Some(3)
Console.WriteLn('abc'.LastIndexOf('z')); // None
```

## See also

- [Str overview](README.md)
- [Split and join](split-join.md)
- [Text index](../../../std/text/README.md)
