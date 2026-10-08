# String operations

String helpers: measure, search, transform, split, and join. This page lists every operation in the native string catalog.

```pascal
program Example;
uses Std.Console;
begin
  WriteLn('hello'.Length());
end.
```

## Availability and call forms

These built-in type operations are always available without imports. Use
dot calls with positional or fully named explicit arguments. The static
receiver type and operation name select one catalog entry; ordinary free
functions with the same name do not affect this selection.

## Quick reference

| Operation | Result / behavior |
| --- | --- |
| `Text.Length(): integer` | Number of Unicode scalars |
| `Text.IsEmpty(): boolean` | Whether the scalar count is zero |
| `Text.Contains(Sub: string): boolean` | Substring membership |
| `Text.StartsWith(Pre: string): boolean` | Prefix test |
| `Text.EndsWith(Suf: string): boolean` | Suffix test |
| `Text.IndexOf(Sub: string): integer` | First scalar index, or `-1` |
| `Text.Trim(): string` | Remove leading and trailing whitespace |
| `Text.ToUpper(): string` | Uppercased value |
| `Text.ToLower(): string` | Lowercased value |
| `Text.Replace(Old: string; New: string): string` | Replace all occurrences |
| `Text.Split(Delim: string): array of string` | Split into segments |
| `Text.Map(F: function(C: string): string): string` | Transform each scalar into exactly one scalar |
| `Text.Filter(F: function(C: string): boolean): string` | Keep matching scalars |
| `Text.Reduce(Init: U; F: function(Acc: U; C: string): U): U` | Fold scalars left to right |
| `Text.Slice(Start: integer; Len: integer): string` | Return the checked scalar range; same public name and argument roles as array `Slice` |
| `Text.LastIndexOf(Sub: string): integer` | Last matching scalar index, or `-1` |
| `Text.IsNumeric(): boolean` | Whether the value matches the existing numeric-text rules |
| `Text.RepeatStr(N: integer): string` | Repeat the complete string |
| `Text.PadLeft(Width: integer; PadChar: string): string` | Pad on the left |
| `Text.PadRight(Width: integer; PadChar: string): string` | Pad on the right |
| `Text.PadCenter(Width: integer; PadChar: string): string` | Center within the padded width |
| `Text.FromChar(N: integer): string` | Repeat a receiver that must contain exactly one scalar |
| `Text.CharAt(Index: integer): string` | Read the scalar at the checked index |
| `Text.SetCharAt(Index: integer; C: string): string` | Return a value with one scalar replaced |
| `Text.Ord(): integer` | Unicode codepoint of a receiver that must contain exactly one scalar |
| `Text.Insert(Index: integer; Sub: string): string` | Insert text at the scalar index |
| `Text.Delete(Index: integer; Len: integer): string` | Remove the checked scalar range |
| `Text.Reverse(): string` | Return reversed scalars |
| `Text.TrimLeft(): string` | Remove leading whitespace |
| `Text.TrimRight(): string` | Remove trailing whitespace |
| `Text.Format(Arguments...): string` | Use the receiver as the format template; heterogeneous variadic arguments are positional only |
| `string.Chr(N: integer): string` | Construct one scalar from a valid Unicode codepoint |

`Length()` and `IsEmpty()` use the same canonical names for strings, arrays, and dictionaries.

`Format(Arguments...)` is positional only. Its string receiver supplies the template.


## Topics

| Topic | Description |
|-------|-------------|
| [Case and trim](case-trim.md) | `Length`, `ToUpper`, `ToLower`, trim |
| [Search](search.md) | `Contains`, `IndexOf`, `Slice`, … |
| [Split and join](split-join.md) | `Split`, `Join` |
| [Edit](edit.md) | `Replace`, `Pad*`, `Insert`, `Delete`, … |
| [Format and characters](format-chars.md) | `Format`, `Ord`, `Chr`, `IsNumeric` |
| [Higher-order operations](higher-order.md) | `Map`, `Filter`, `Reduce` over Unicode scalars |

## Implementation (contributors)

Search predicates, `IndexOf`, `LastIndexOf`, `IsNumeric`, and `CharAt` borrow
their input strings. They do not copy the complete input before querying it.
Character indices retain their Unicode scalar semantics.

`Slice` validates against the cached scalar count and copies only the selected
UTF-8 range. ASCII ranges use byte offsets directly; Unicode ranges scan scalar
boundaries without allocating a character vector. A full-range result shares the
immutable input storage.

| Concern | Location |
|---------|-----------|
| Algorithms | [`str/mod.rs`](../../../../../crates/fpas-std/src/str/mod.rs) |
| Scalar substring ranges | [`str/substring.rs`](../../../../../crates/fpas-std/src/str/substring.rs) |
| Shared string storage (`SharedStr`, cached `char_len` for O(1) `Length`) | [`value/mod.rs`](../../../../../crates/fpas-bytecode/src/value/mod.rs) |
| String concatenation (sums cached lengths) | [`scalar.rs`](../../../../../crates/fpas-vm/src/vm/value_ops/scalar.rs) |
| Higher-order callbacks | [`hosted/callbacks/`](../../../../../crates/fpas-vm/src/vm/hosted/callbacks) |
| Registration | [`std_registry/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Text and parsing index](../../../std/text/README.md)
- [Standard library index](../../../std/README.md)
