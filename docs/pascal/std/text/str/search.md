# Search

## `function Contains(S: string; Sub: string): boolean`

`true` if `Sub` occurs anywhere in `S`, else `false`.

```pascal
if Contains('abc', 'b') then
  WriteLn('yes')
```

---

## `function StartsWith(S: string; Pre: string): boolean`

`true` if `S` begins with `Pre`.

```pascal
WriteLn(StartsWith('abc', 'ab'))
```

---

## `function EndsWith(S: string; Suf: string): boolean`

`true` if `S` ends with `Suf`.

```pascal
WriteLn(EndsWith('abc', 'bc'))
```

---

## `function Substring(S: string; Start: integer; Len: integer): string`

Copies `Len` characters starting at `Start`. **Bounds are checked at runtime**; invalid ranges produce a runtime error.

```pascal
WriteLn(Substring('Hello', 0, 3))
```

---

## `function IndexOf(S: string; Sub: string): integer`

Returns the **first** character index of `Sub` in `S`, or **`-1`** if not found.

```pascal
WriteLn(IndexOf('aba', 'a'));
WriteLn(IndexOf('aba', 'z'))
```

---

## `function LastIndexOf(S: string; Sub: string): integer`

Returns the **last** character index of `Sub` in `S`, or **`-1`** if not found.

```pascal
WriteLn(LastIndexOf('abcabc', 'abc'))  // 3
WriteLn(LastIndexOf('abc', 'z'))       // -1
```

## See also

- [Str overview](README.md)
- [Split and join](split-join.md)
- [Text index](../README.md)
