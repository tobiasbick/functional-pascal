# Edit

## `Text.Replace(Old: string; New: string): string`

Replaces **all** non-overlapping occurrences of `Old` with `New`.

```pascal
WriteLn('aaa'.Replace('a', 'b'));
```

---

## `Text.RepeatStr(N: integer): string`

Returns `S` concatenated `Count` times. `Count` ≤ 0 yields an empty string.


```pascal
WriteLn('ab'.RepeatStr(3));  // ababab
WriteLn('─'.RepeatStr(40)); // ────────────────────────────────────────
```

`Count` must be at most **1_000_000** when positive. Larger counts raise a runtime error instead of allocating unbounded memory.

---

## `Text.PadLeft(Width: integer; PadChar: string): string`

If `S.Length() < Width`, prepends `Fill` characters until length equals `Width`. Otherwise returns `S` unchanged.

`Width` must be at most **1_000_000**. Larger widths raise a runtime error.

```pascal
WriteLn('42'.PadLeft(5, '0'));  // 00042
```

---

## `Text.PadRight(Width: integer; PadChar: string): string`

Like `PadLeft` but appends `Fill` on the right.

`Width` must be at most **1_000_000**. Larger widths raise a runtime error.

```pascal
WriteLn('Hi'.PadRight(6, '.'));  // Hi....
```

---

## `Text.PadCenter(Width: integer; PadChar: string): string`

Centers `S` within `Width` characters of `Fill`. When the remaining space is odd, the extra character goes on the right.

`Width` must be at most **1_000_000**. Larger widths raise a runtime error.

```pascal
WriteLn('Hi'.PadCenter(6, '-'));  // --Hi--
```

---

## `Text.FromChar(N: integer): string`

Builds a string of `Count` copies of `C`. `C` must contain exactly one Unicode scalar value; empty or multi-character strings are runtime errors. `Count` ≤ 0 yields an empty string.

Positive `Count` must be at most **1_000_000**. Larger counts raise a runtime error.

```pascal
WriteLn('─'.FromChar(40));
```

---

## `Text.CharAt(Index: integer): string`

Returns the character at the 0-based `Index`. **Runtime error** if out of bounds.

Indices count Unicode scalars. For a nonempty string, valid indices are
`0..S.Length()-1`. Empty strings have no valid character index. Invalid indices
report FP5021; the hint uses `S.Length()` for nonempty strings and recommends
checking `S.IsEmpty()` for empty strings.

```pascal
const C: string := 'Hello'.CharAt(0);
WriteLn(C);  // H
```

---

## `Text.SetCharAt(Index: integer; C: string): string`

Returns a **new** string that is identical to `S` except the character at `Index` is replaced with `C`. `C` must contain exactly one Unicode scalar value. **Runtime error** if `Index` is out of bounds or `C` is empty or contains multiple characters.

`Index` follows the same scalar bounds and empty-string checks as `CharAt`.

```pascal
WriteLn('Hello'.SetCharAt(0, 'J'));  // Jello
```

---

## `Text.Ord(): integer`

Returns the Unicode codepoint (integer value) of `C`. `C` must contain exactly one Unicode scalar value; empty or multi-character strings are runtime errors.

```pascal
WriteLn('A'.Ord());  // 65
```

---

## `string.Chr(N: integer): string`

Returns the character with Unicode codepoint `N`. **Runtime error** if `N` is not a valid Unicode scalar value.

```pascal
WriteLn(string.Chr(65));  // A
```

---

## `Text.Insert(Index: integer; Sub: string): string`

Returns a new string with `Sub` inserted at position `Index`. **Runtime error** if `Index` is out of range `[0..S.Length()]`.

Indices count Unicode scalars, and `S.Length()` is a valid insertion position.
For an empty string, index `0` is valid. Invalid indices report FP5021 with a
hint naming the inclusive `0..S.Length()` range.

```pascal
WriteLn('Hllo'.Insert(1, 'e'));  // Hello
```

---

## `Text.Delete(Index: integer; Len: integer): string`

Returns a new string with `Len` characters removed starting at `Start`. **Runtime error** if the range is out of bounds.

```pascal
WriteLn('Hello'.Delete(1, 3));  // Ho
```

---

## `Text.Reverse(): string`

Returns a new string with characters in reverse order.

```pascal
WriteLn('abc'.Reverse());  // cba
```

## See also

- [Str overview](README.md)
- [Format and characters](format-chars.md)
- [Text index](../../../std/text/README.md)
