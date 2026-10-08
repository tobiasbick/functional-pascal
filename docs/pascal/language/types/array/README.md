# Array operations

Non-mutating array helpers (length, sort, search, slice, …) plus **in-place** `Push` and `Pop`. This page lists every operation in the native array catalog.

```pascal
program Example;
uses Std.Console;
begin
  const A: array of integer := [1, 2, 3];
  WriteLn(A.Length());
end.
```

## Availability and call forms

These built-in type operations are always available without imports. Use
dot calls with positional or fully named explicit arguments. The static
receiver type and operation name select one catalog entry; ordinary free
functions with the same name do not affect this selection.

---

## Quick reference

| Operation | Result / behavior |
| --- | --- |
| `Items.Length(): integer` | Number of elements |
| `Items.IsEmpty(): boolean` | Whether the element count is zero |
| `Items.Contains(Value: T): boolean` | Element membership |
| `Items.IndexOf(Value: T): integer` | First matching index, or `-1` |
| `Items.Map(F: function(X: T): U): array of U` | Transform each element |
| `Items.Filter(F: function(X: T): boolean): array of T` | Keep matching elements |
| `Items.Reduce(Init: U; F: function(Acc: U; V: T): U): U` | Fold elements left to right |
| `Items.Find(F: function(X: T): boolean): Option of T` | First matching element, or `None` |
| `Items.FindIndex(F: function(X: T): boolean): integer` | First matching index, or `-1` |
| `Items.Any(F: function(X: T): boolean): boolean` | Whether any element satisfies the predicate |
| `Items.All(F: function(X: T): boolean): boolean` | Whether every element satisfies the predicate |
| `Items.Sort(): array of T` | Return a sorted value |
| `Items.Reverse(): array of T` | Return a reversed value |
| `Items.Slice(Start: integer; Len: integer): array of T` | Return the checked subrange |
| `Items.Concat(B: array of T): array of T` | Append the second array's elements to the result |
| `Items.FlatMap(F: function(X: T): array of U): array of U` | Map each element, then flatten the results |
| `Items.ForEach(F: procedure(X: T))` | Invoke a procedure per element; only a final chain step |
| `Items.Push(Value: T)` | Append to the caller's array; requires a writable receiver, with no call-site receiver marker |
| `Items.Pop(): T` | Remove and return the last element; requires a writable receiver, with no call-site receiver marker |
| `Items.Join(Delim: string): string` | Join the receiver's strings with the delimiter |
| `array.Fill(Value: T; Count: integer): array of T` | Construct an array of repeated values |

`Length()` and `IsEmpty()` use the same canonical names for strings, arrays, and dictionaries.


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

- [Collections index](../../../std/collections/README.md)
- [Standard library index](../../../std/README.md)
