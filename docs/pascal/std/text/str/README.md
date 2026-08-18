# `Std.Str`

String helpers: measure, search, transform, split, and join. This page lists **every** exported symbol; you do not need the implementation source to use the unit.

```pascal
program Example;
uses Std.Console, Std.Str;
begin
  WriteLn(Length('hello'))
end.
```

## Importing and names

After `uses Std.Str;` you may use **short** names (`Length`, `ToUpper`, …) or **qualified** names (`Std.Str.Length`, …).

**Ambiguity:** if you also `uses Std.Array`, the short names **`Length`**, **`Contains`**, and **`IndexOf`** exist in both units. The compiler reports an **ambiguous** error unless you qualify, for example `Std.Str.Length(S)` vs `Std.Array.Length(A)`.

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
| function | `RepeatStr(S: string; Count: integer): string` | repeat `S`; `Count <= 0` returns empty string |
| function | `PadLeft(S: string; Width: integer; Fill: string): string` | left-pad to `Width` |
| function | `PadRight(S: string; Width: integer; Fill: string): string` | right-pad to `Width` |
| function | `PadCenter(S: string; Width: integer; Fill: string): string` | center-pad to `Width` |
| function | `FromChar(C: string; Count: integer): string` | repeated one-scalar string |
| function | `CharAt(S: string; Index: integer): string` | character at 0-based index |
| function | `SetCharAt(S: string; Index: integer; C: string): string` | replace with one-scalar string |
| function | `Ord(C: string): integer` | codepoint of one-scalar string |
| function | `Chr(N: integer): string` | character from codepoint |
| function | `Insert(S: string; Index: integer; Sub: string): string` | insert `Sub` at index |
| function | `Delete(S: string; Start: integer; Len: integer): string` | remove `Len` chars |
| function | `Reverse(S: string): string` | reversed copy |
| function | `TrimLeft(S: string): string` | strip leading whitespace |
| function | `TrimRight(S: string): string` | strip trailing whitespace |
| function | `LastIndexOf(S: string; Sub: string): integer` | last index or `-1` |
| function | `Format(Template: string; ...): string` | printf-style string formatting |

**Indexing:** all “character index” parameters are in **Unicode scalar** units (user-visible characters), not UTF-8 bytes.

**Variadic formatting:** `Format` always requires the first argument `Template: string`. Each specifier other than `%%` consumes exactly one additional argument. Placeholder compatibility is validated at runtime.

## Topics

| Topic | Description |
|-------|-------------|
| [Case and trim](case-trim.md) | `Length`, `ToUpper`, `ToLower`, trim |
| [Search](search.md) | `Contains`, `IndexOf`, `Substring`, … |
| [Split and join](split-join.md) | `Split`, `Join` |
| [Edit](edit.md) | `Replace`, `Pad*`, `Insert`, `Delete`, … |
| [Format and characters](format-chars.md) | `Format`, `Ord`, `Chr`, `IsNumeric` |

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Algorithms | [`str.rs`](../../../../../crates/fpas-std/src/str.rs) |
| Shared string storage (`SharedStr`, cached `char_len` for O(1) `Length`) | [`value/mod.rs`](../../../../../crates/fpas-bytecode/src/value/mod.rs) |
| String concatenation (sums cached lengths) | [`scalar.rs`](../../../../../crates/fpas-vm/src/vm/value_ops/scalar.rs) |
| Registration | [`std_registry/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Text and parsing index](../README.md)
- [Standard library index](../../README.md)
