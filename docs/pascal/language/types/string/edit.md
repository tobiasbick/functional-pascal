# Edit

## `Text.Replace(Old: string; New: string): string`

Replaces **all** non-overlapping occurrences of `Old` with `New`.

```pascal
Console.WriteLn('aaa'.Replace('a', 'b'));
```

---

## `Text.RepeatStr(N: integer): string`

Returns `S` concatenated `Count` times. `Count` ≤ 0 yields an empty string.
Empty, single-scalar, and multi-scalar strings are all valid receivers.
The receiver is preserved; `RepeatStrInPlace` replaces writable storage.
User APIs needing exactly one scalar declare an explicit precondition such as
`requires Text.Length() = 1;` before using repetition.

```pascal
Console.WriteLn('ab'.RepeatStr(3)); // ababab
Console.WriteLn('─'.RepeatStr(40)); // ────────────────────────────────────────
```

`Count` must be at most **1_000_000** when positive. Larger counts raise a runtime error instead of allocating unbounded memory.

---

## `Text.PadLeft(Width: integer; PadChar: string): string`

If `Text.Length() < Width`, prepends copies of the first Unicode scalar in
`PadChar` until length equals `Width`. Otherwise returns `Text` unchanged after
validating the arguments.

All three padding operations require a nonempty `PadChar`; an empty string
raises FP5008 even when no padding is needed. Only the first Unicode scalar is
used; additional characters are ignored.

`Width` must be at most **1_000_000**. Larger widths raise a runtime error.

```pascal
Console.WriteLn('42'.PadLeft(5, '0')); // 00042
```

---

## `Text.PadRight(Width: integer; PadChar: string): string`

Like `PadLeft` but appends padding on the right. The same nonempty `PadChar`
validation and first-scalar rules apply, including when no padding is needed.

`Width` must be at most **1_000_000**. Larger widths raise a runtime error.

```pascal
Console.WriteLn('Hi'.PadRight(6, '.')); // Hi....
```

---

## `Text.PadCenter(Width: integer; PadChar: string): string`

Centers `Text` within `Width` characters using the first Unicode scalar in
`PadChar`. When the remaining space is odd, the extra character goes on the
right. The same nonempty `PadChar` validation applies even when no padding is needed.

`Width` must be at most **1_000_000**. Larger widths raise a runtime error.

```pascal
Console.WriteLn('Hi'.PadCenter(6, '-')); // --Hi--
```

---

## Scalar indexing: `Text[Index]`

Returns the character at the 0-based `Index`. **Runtime error** if out of bounds.

Indices count Unicode scalars. For a nonempty string, valid indices are
`0..S.Length()-1`. Empty strings have no valid character index. Invalid indices
raise a runtime error. Check `S.IsEmpty()` before reading an empty string.

```pascal
const C: string := 'Hello'[0];
Console.WriteLn(C); // H
```

---

## `Text.SetCharAt(Index: integer; C: string): string`

Returns a **new** string that is identical to `S` except the character at `Index` is replaced with `C`. `C` must contain exactly one Unicode scalar value. **Runtime error** if `Index` is out of bounds or `C` is empty or contains multiple characters.

`Index` follows the same scalar bounds and empty-string checks as scalar indexing.

```pascal
Console.WriteLn('Hello'.SetCharAt(0, 'J')); // Jello
```

---

## `Text.Ord(): integer`

Returns the Unicode codepoint (integer value) of `C`. `C` must contain exactly one Unicode scalar value; empty or multi-character strings are runtime errors.

```pascal
Console.WriteLn('A'.Ord()); // 65
```

---

## `string.Chr(N: integer): string`

Returns the character with Unicode codepoint `N`. **Runtime error** if `N` is not a valid Unicode scalar value.

```pascal
Console.WriteLn(string.Chr(65)); // A
```

---

## `Text.Insert(Index: integer; Sub: string): string`

Returns a new string with `Sub` inserted at position `Index`. **Runtime error** if `Index` is out of range `[0..S.Length()]`.

Indices count Unicode scalars, and `S.Length()` is a valid insertion position.
For an empty string, index `0` is valid. Invalid indices report FP5021 with a
hint naming the inclusive `0..S.Length()` range.

```pascal
Console.WriteLn('Hllo'.Insert(1, 'e')); // Hello
```

---

## `Text.Delete(Index: integer; Len: integer): string`

Returns a new string with `Len` characters removed starting at `Start`. **Runtime error** if the range is out of bounds.

```pascal
Console.WriteLn('Hello'.Delete(1, 3)); // Ho
```

---

## `Text.Reverse(): string`

Returns a new string with characters in reverse order.

```pascal
Console.WriteLn('abc'.Reverse()); // cba
```

## See also

- [Str overview](README.md)
- [Format and characters](format-chars.md)
- [Text index](../../../std/text/README.md)
