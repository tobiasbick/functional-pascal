# `Std.Arrays`

Non-mutating array helpers (length, sort, search, slice, …) plus **in-place** `Push` and `Pop`. This page lists the **entire** surface of the unit.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Arrays as Arrays;

begin
  const A: array of (integer) := [1, 2, 3];
  Console.WriteLn(Arrays.Length(A));
end program;
```

## Importing and names

Import with `uses Std.Arrays as Arrays;`. Access every exported member through `Arrays`, for example `Arrays.Length(...)`. Imports open no short names.

Explicit aliases keep names from different units distinct. Imported routines use alias-qualified calls.

---

## Quick reference

All routines are **generic over element type `T`** (your array’s element type).

| Kind | Name | Notes |
|------|------|--------|
| function | `Length(A: array of (T)): integer` | element count |
| function | `Sort(A: array of (T)): array of (T)` | new sorted array |
| function | `Reverse(A: array of (T)): array of (T)` | new reversed array |
| function | `Contains(A: array of (T); Value: T): boolean` | membership |
| function | `IndexOf(A: array of (T); Value: T): integer` | first index or `-1` |
| function | `Slice(A: array of (T); Start: integer; Len: integer): array of (T)` | sub-range; bounds checked |
| procedure | `Push(var A: array of (T); Value: T)` | append in place |
| function | `Pop(var A: array of (T)): T` | remove last |
| function | `Map(A: array of (T); F: pure function(X: T): U): array of (U)` | transform each element |
| function | `Filter(A: array of (T); F: pure function(X: T): boolean): array of (T)` | keep matching elements |
| function | `Reduce(A: array of (T); Init: U; F: pure function(Acc: U; V: T): U): U` | fold to single value |
| function | `Find(A: array of (T); F: pure function(X: T): boolean): Option of (T)` | first match or `Option.None` |
| function | `FindIndex(A: array of (T); F: pure function(X: T): boolean): integer` | index of first match or `-1` |
| function | `Any(A: array of (T); F: pure function(X: T): boolean): boolean` | `true` if any satisfies `F` |
| function | `All(A: array of (T); F: pure function(X: T): boolean): boolean` | `true` if all satisfy `F` |
| function | `Concat(A: array of (T); B: array of (T)): array of (T)` | concatenate two arrays |
| function | `FlatMap(A: array of (T); F: pure function(X: T): array of (U)): array of (U)` | map then flatten |
| function | `Fill(Value: T; Count: integer): array of (T)` | array of `Count` copies |
| procedure | `ForEach(A: array of (T); F: procedure(X: T))` | call `F` for each element |

**Mutating calls:** `Push` and `Pop` require an explicit `var A` argument.
The target may select stored fields or elements rooted in a mutable binding or a
forwarded `var` parameter. See [mutation rules](mutating.md).

**Purity:** functions in this unit are pure and require resource-free data or pure
callables throughout their arguments and results. Functional callbacks must be
explicitly pure, for example `F: pure function(X: T): boolean`. `Push`, `Pop`, and
`ForEach` are ordinary procedures; `ForEach` accepts an ordinary procedure callback.

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
| `Push` / `Pop` | [`vm/mod.rs`](../../../../../crates/fpas-vm/src/vm/mod.rs), [`lowering/calls.rs`](../../../../../crates/fpas-compiler/src/lowering/calls.rs) |
| Registration | [`std_registry/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Collections index](../README.md)
- [Standard library index](../../README.md)
