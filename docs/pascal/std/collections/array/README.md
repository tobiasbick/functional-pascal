# `Std.Array`

Non-mutating array helpers (length, sort, search, slice, …) plus **in-place** `Push` and `Pop`. This page lists the **entire** surface of the unit.

```pascal
program Example;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Length(A))
end.
```

## Importing and names

After `uses Std.Array;` use short names (`Length`, `Sort`, …) or qualified (`Std.Array.Length`, …).

**Ambiguity with `Std.Str`:** short names **`Length`**, **`Contains`**, and **`IndexOf`** clash. Qualify as `Std.Array.Length(A)` vs `Std.Str.Length(S)`, etc.

---

## Quick reference

All routines are **generic over element type `T`** (your array’s element type).

| Kind | Name | Notes |
|------|------|--------|
| function | `Length(A: array of T): integer` | element count |
| function | `Sort(A: array of T): array of T` | new sorted array |
| function | `Reverse(A: array of T): array of T` | new reversed array |
| function | `Contains(A: array of T; Value: T): boolean` | membership |
| function | `IndexOf(A: array of T; Value: T): integer` | first index or `-1` |
| function | `Slice(A: array of T; Start: integer; Len: integer): array of T` | sub-range; bounds checked |
| procedure | `Push(mutable A: array of T; Value: T)` | append in place |
| function | `Pop(mutable A: array of T): T` | remove last |
| function | `Map(A: array of T; F: function(X: T): U): array of U` | transform each element |
| function | `Filter(A: array of T; F: function(X: T): boolean): array of T` | keep matching elements |
| function | `Reduce(A: array of T; Init: U; F: function(Acc: U; V: T): U): U` | fold to single value |
| function | `Find(A: array of T; F: function(X: T): boolean): Option of T` | first match or `None` |
| function | `FindIndex(A: array of T; F: function(X: T): boolean): integer` | index of first match or `-1` |
| function | `Any(A: array of T; F: function(X: T): boolean): boolean` | `true` if any satisfies `F` |
| function | `All(A: array of T; F: function(X: T): boolean): boolean` | `true` if all satisfy `F` |
| function | `Concat(A: array of T; B: array of T): array of T` | concatenate two arrays |
| function | `FlatMap(A: array of T; F: function(X: T): array of U): array of U` | map then flatten |
| function | `Fill(Value: T; Count: integer): array of T` | array of `Count` copies |
| procedure | `ForEach(A: array of T; F: procedure(X: T))` | call `F` for each element |

**Mutating calls:** `Push` and `Pop` require **`A` to be a simple mutable array variable** (typically `mutable var Name: array of T := …`). The compiler rejects other targets.

**Callbacks:** pass a named function or procedure whose type matches the parameter (e.g. `F: function(X: T): boolean`).

## Topics

| Topic | Description |
|-------|-------------|
| [Basics](basics.md) | `Length`, `Sort`, `Reverse`, search, `Slice` |
| [Mutating](mutating.md) | `Push`, `Pop` |
| [Higher-order](higher-order.md) | `Map`, `Filter`, `Reduce`, `Find`, `Any`, `All` |
| [Combine and iterate](combine.md) | `Concat`, `FlatMap`, `Fill`, `ForEach` |

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Pure helpers | [`array.rs`](../../../../../crates/fpas-std/src/array.rs) |
| `Push` / `Pop` | [`vm/mod.rs`](../../../../../crates/fpas-vm/src/vm/mod.rs), [`std_calls/array.rs`](../../../../../crates/fpas-compiler/src/compiler/std_calls/array.rs) |
| Registration | [`std_registry/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Collections index](../README.md)
- [Standard library index](../../README.md)
