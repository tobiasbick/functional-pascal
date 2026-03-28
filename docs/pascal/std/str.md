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

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Algorithms | [`str.rs`](../../../crates/fpas-std/src/str.rs) |
| Registration | [`std_registry.rs`](../../../crates/fpas-sema/src/std_registry.rs) |

[← Standard library index](README.md)
