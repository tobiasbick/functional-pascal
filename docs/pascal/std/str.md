# `Std.Str`

String helpers: measure, search, transform, split, and join. This page lists **every** exported symbol; you do not need the implementation source to use the unit.

```pascal
program Example;
uses Std.Console, Std.Str;
begin
  WriteLn(Length('hello'))
end.
```

**Maintenance (implementers only):** keep aligned with [`std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs), [`str.rs`](../../../crates/fpas-std/src/str.rs), [`intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs), [`intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs), and [`compiler.rs`](../../../crates/fpas-compiler/src/compiler.rs).

---

## Importing and names

After `uses Std.Str;` you may use **short** names (`Length`, `ToUpper`, …) or **qualified** names (`Std.Str.Length`, …).

**Ambiguity:** if you also `uses Std.Array`, the short names **`Length`**, **`Contains`**, and **`IndexOf`** exist in both units. The compiler reports an **ambiguous** error unless you qualify, for example `Std.Str.Length(S)` vs `Std.Array.Length(A)`.

---

## Quick reference

Requires `uses Std.Str;`.

| Kind | Name | Result |
|------|------|--------|
| function | `Length(S: string): integer` | character count |
| function | `ToUpper(S: string): string` | uppercased copy |
| function | `ToLower(S: string): string` | lowercased copy |
| function | `Trim(S: string): string` | trim whitespace |
| function | `Contains(S: string; Sub: string): boolean` | substring test |
| function | `StartsWith(S: string; Pre: string): boolean` | prefix test |
| function | `EndsWith(S: string; Suf: string): boolean` | suffix test |
| function | `Substring(S: string; Start: integer; Len: integer): string` | slice by index |
| function | `IndexOf(S: string; Sub: string): integer` | first index or `-1` |
| function | `Replace(S: string; Old: string; New: string): string` | replace all |
| function | `Split(S: string; Delim: string): array of string` | split segments |
| function | `Join(Parts: array of string; Delim: string): string` | join with delimiter |
| function | `IsNumeric(S: string): boolean` | parses as number? |
| function | `RepeatStr(S: string; Count: integer): string` | repeat `S` exactly `Count` times |
| function | `PadLeft(S: string; Width: integer; Fill: char): string` | left-pad to `Width` |
| function | `PadRight(S: string; Width: integer; Fill: char): string` | right-pad to `Width` |
| function | `PadCenter(S: string; Width: integer; Fill: char): string` | center-pad to `Width` |
| function | `FromChar(C: char; Count: integer): string` | string of repeated char |
| function | `CharAt(S: string; Index: integer): char` | character at 0-based index |
| function | `SetCharAt(S: string; Index: integer; C: char): string` | new string with one char replaced |
| function | `Ord(C: char): integer` | Unicode codepoint |
| function | `Chr(N: integer): char` | character from codepoint |
| function | `Insert(S: string; Index: integer; Sub: string): string` | insert `Sub` at index |
| function | `Delete(S: string; Start: integer; Len: integer): string` | remove `Len` chars |
| function | `Reverse(S: string): string` | reversed copy |
| function | `TrimLeft(S: string): string` | strip leading whitespace |
| function | `TrimRight(S: string): string` | strip trailing whitespace |
| function | `LastIndexOf(S: string; Sub: string): integer` | last index or `-1` |
| function | `Format(Template: string; ...): string` | printf-style string formatting |
**Indexing:** all “character index” parameters are in **Unicode scalar** units (user-visible characters), not UTF-8 bytes.

---

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

## `function Replace(S: string; Old: string; New: string): string`

Replaces **all** non-overlapping occurrences of `Old` with `New`.

```pascal
WriteLn(Replace('aaa', 'a', 'b'))
```

---

## `function Split(S: string; Delim: string): array of string`

Splits `S` around each occurrence of `Delim`. Returns a new array of segments.

- **`Delim` must not be empty** — empty delimiter is a **runtime error**.

```pascal
program SplitDemo;
uses Std.Console, Std.Str, Std.Array;
begin
  var Parts: array of string := Split('x,y', ',');
  WriteLn(Std.Array.Length(Parts))
end.
```

(`Length` for arrays would be ambiguous with `Std.Str` also imported; qualify `Std.Array.Length` here.)

---

## `function Join(Parts: array of string; Delim: string): string`

Concatenates every element of `Parts`, inserting `Delim` between elements.

```pascal
WriteLn(Join(['x', 'y'], ':'))
```

---

## `function IsNumeric(S: string): boolean`

`true` if the string (after trim) parses as an **integer** or **real**, otherwise `false`.

```pascal
WriteLn(IsNumeric('42'));
WriteLn(IsNumeric('nope'))
```

---

## `function RepeatStr(S: string; Count: integer): string`

Returns `S` concatenated `Count` times. `Count` ≤ 0 yields an empty string.

> **Note:** `Repeat` cannot be used as a short name because `repeat` is a reserved keyword (`repeat … until` loop). Use the qualified form `Std.Str.RepeatStr` or the short name `RepeatStr`.

```pascal
WriteLn(RepeatStr('ab', 3))  { ababab }
WriteLn(RepeatStr('─', 40)) { ────────────────────────────────────────}
```

---

## `function PadLeft(S: string; Width: integer; Fill: char): string`

If `Length(S) < Width`, prepends `Fill` characters until length equals `Width`. Otherwise returns `S` unchanged.

```pascal
WriteLn(PadLeft('42', 5, '0'))  { 00042 }
```

---

## `function PadRight(S: string; Width: integer; Fill: char): string`

Like `PadLeft` but appends `Fill` on the right.

```pascal
WriteLn(PadRight('Hi', 6, '.'))  { Hi.... }
```

---

## `function PadCenter(S: string; Width: integer; Fill: char): string`

Centers `S` within `Width` characters of `Fill`. When the remaining space is odd, the extra character goes on the right.

```pascal
WriteLn(PadCenter('Hi', 6, '-'))  { --Hi-- }
```

---

## `function FromChar(C: char; Count: integer): string`

Builds a string of `Count` copies of `C`. `Count` ≤ 0 yields an empty string.

```pascal
WriteLn(FromChar('─', 40))
```

---

## `function CharAt(S: string; Index: integer): char`

Returns the character at the 0-based `Index`. **Runtime error** if out of bounds.

```pascal
var C: char := CharAt('Hello', 0);
WriteLn(C)  { H }
```

---

## `function SetCharAt(S: string; Index: integer; C: char): string`

Returns a **new** string that is identical to `S` except the character at `Index` is replaced with `C`. **Runtime error** if out of bounds.

```pascal
WriteLn(SetCharAt('Hello', 0, 'J'))  { Jello }
```

---

## `function Ord(C: char): integer`

Returns the Unicode codepoint (integer value) of `C`.

```pascal
WriteLn(Ord('A'))  { 65 }
```

---

## `function Chr(N: integer): char`

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

---

## `function LastIndexOf(S: string; Sub: string): integer`

Returns the **last** character index of `Sub` in `S`, or **`-1`** if not found.

```pascal
WriteLn(LastIndexOf('abcabc', 'abc'))  { 3 }
WriteLn(LastIndexOf('abc', 'z'))       { -1 }
```

---

## `function Format(Template: string; ...): string`

Returns a new string by substituting format specifiers in `Template` with the supplied arguments.

```pascal
var Status: string := Format('Zoom: %fx Center: (%f, %f)', Zoom, CX, CY);
var Msg: string    := Format('Item %d: %s', Index, Name);
var Pct: string    := Format('100%%');  { '100%' }
```

### Specifiers

| Specifier | Accepted type | Example |
|-----------|--------------|---------|
| `%d` | `integer` | `Format('%d', 42)` → `'42'` |
| `%f` | `real` or `integer` | `Format('%f', 3.14)` → `'3.14'` |
| `%s` | `string` or `char` | `Format('%s', 'hi')` → `'hi'` |
| `%%` | *(no argument)* | `Format('100%%')` → `'100%'` |

The number of specifiers (excluding `%%`) must exactly match the number of extra arguments; a mismatch is a **runtime error**.

---

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Algorithms | [`str.rs`](../../../crates/fpas-std/src/str.rs) |
| Registration | [`std_registry.rs`](../../../crates/fpas-sema/src/std_registry.rs) |

[← Standard library index](README.md)
