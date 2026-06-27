# Edit

## `function Replace(S: string; Old: string; New: string): string`

Replaces **all** non-overlapping occurrences of `Old` with `New`.

```pascal
WriteLn(Replace('aaa', 'a', 'b'))
```

---

## `function RepeatStr(S: string; Count: integer): string`

Returns `S` concatenated `Count` times. `Count` ≤ 0 yields an empty string.

> **Note:** After `uses Std.Str`, call this routine as `RepeatStr` (the name `Repeat` is reserved for the `repeat … until` loop).

```pascal
WriteLn(RepeatStr('ab', 3))  { ababab }
WriteLn(RepeatStr('─', 40)) { ────────────────────────────────────────}
```

`Count` must be at most **1_000_000** when positive. Larger counts raise a runtime error instead of allocating unbounded memory.

---

## `function PadLeft(S: string; Width: integer; Fill: string): string`

If `Length(S) < Width`, prepends `Fill` characters until length equals `Width`. Otherwise returns `S` unchanged.

```pascal
WriteLn(PadLeft('42', 5, '0'))  { 00042 }
```

---

## `function PadRight(S: string; Width: integer; Fill: string): string`

Like `PadLeft` but appends `Fill` on the right.

```pascal
WriteLn(PadRight('Hi', 6, '.'))  { Hi.... }
```

---

## `function PadCenter(S: string; Width: integer; Fill: string): string`

Centers `S` within `Width` characters of `Fill`. When the remaining space is odd, the extra character goes on the right.

```pascal
WriteLn(PadCenter('Hi', 6, '-'))  { --Hi-- }
```

---

## `function FromChar(C: string; Count: integer): string`

Builds a string of `Count` copies of `C`. `Count` ≤ 0 yields an empty string.

```pascal
WriteLn(FromChar('─', 40))
```

---

## `function CharAt(S: string; Index: integer): string`

Returns the character at the 0-based `Index`. **Runtime error** if out of bounds.

```pascal
var C: string := CharAt('Hello', 0);
WriteLn(C)  { H }
```

---

## `function SetCharAt(S: string; Index: integer; C: string): string`

Returns a **new** string that is identical to `S` except the character at `Index` is replaced with `C`. **Runtime error** if out of bounds.

```pascal
WriteLn(SetCharAt('Hello', 0, 'J'))  { Jello }
```

---

## `function Ord(C: string): integer`

Returns the Unicode codepoint (integer value) of `C`.

```pascal
WriteLn(Ord('A'))  { 65 }
```

---

## `function Chr(N: integer): string`

Returns the character with Unicode codepoint `N`. **Runtime error** if `N` is not a valid Unicode scalar value.

```pascal
WriteLn(Chr(65))  { A }
```

---

## `function Insert(S: string; Index: integer; Sub: string): string`

Returns a new string with `Sub` inserted at position `Index`. **Runtime error** if `Index` is out of range `[0..Length(S)]`.

```pascal
WriteLn(Insert('Hllo', 1, 'e'))  { Hello }
```

---

## `function Delete(S: string; Start: integer; Len: integer): string`

Returns a new string with `Len` characters removed starting at `Start`. **Runtime error** if the range is out of bounds.

```pascal
WriteLn(Delete('Hello', 1, 3))  { Ho }
```

---

## `function Reverse(S: string): string`

Returns a new string with characters in reverse order.

```pascal
WriteLn(Reverse('abc'))  { cba }
```

## See also

- [Str overview](README.md)
- [Format and characters](format-chars.md)
- [Text index](../README.md)
