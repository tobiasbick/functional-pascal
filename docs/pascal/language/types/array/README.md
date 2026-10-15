# Array operations

Value-producing array operations and explicitly mutating InPlace counterparts.
This page lists every operation in the native array catalog.

```pascal
program Example;

uses Std.Console as Console;

begin
  const A: array of integer := [1, 2, 3];
  Console.WriteLn(A.Length());
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
| `Items.IndexOf(Value: T): Option of integer` | `Some` of first matching index, or `None` |
| `Items.Map(F: function(X: T): U): array of U` | Transform each element |
| `Items.Filter(F: function(X: T): boolean): array of T` | Keep matching elements |
| `Items.Reduce(Init: U; F: function(Acc: U; V: T): U): U` | Fold elements left to right |
| `Items.Find(F: function(X: T): boolean): Option of T` | First matching element, or `None` |
| `Items.FindIndex(F: function(X: T): boolean): Option of integer` | `Some` of first matching index, or `None` |
| `Items.Any(F: function(X: T): boolean): boolean` | Whether any element satisfies the predicate |
| `Items.All(F: function(X: T): boolean): boolean` | Whether every element satisfies the predicate |
| `Items.Sort(): array of T` | Return stable ascending Comparable elements; see [element restrictions](basics.md#itemssort-array-of-t) |
| `Items.SortBy(F: function(X: T): K): array of T` | Return stable ascending order by Comparable keys; evaluate F once per element in input order |
| `Items.Reverse(): array of T` | Return a reversed value |
| `Items.Slice(Start: integer; Len: integer): array of T` | Return the checked subrange |
| `Items.Concat(B: array of T): array of T` | Append the second array's elements to the result |
| `Items.FlatMap(F: function(X: T): array of U): array of U` | Map each element, then flatten the results |
| `Items.ForEach(F: procedure(X: T))` | Invoke a procedure per element; only a final chain step |
| `Items.Push(Value: T): array of T` | Return a new appended array; preserve the receiver |
| `Items.Pop(): ArrayPopResult of T` | Return Remaining and Value; preserve receiver; empty arrays raise FP5003 |
| `Items.SortInPlace()` | Replace writable receiver with stable ascending Comparable elements |
| `Items.SortByInPlace(F: function(X: T): K)` | Replace writable receiver with stable ascending order by Comparable keys |
| `Items.ReverseInPlace()` | Reverse writable receiver |
| `Items.FilterInPlace(F: function(X: T): boolean)` | Replace writable receiver with matching elements |
| `Items.SliceInPlace(Start: integer; Len: integer)` | Replace writable receiver with checked subrange |
| `Items.ConcatInPlace(B: array of T)` | Append argument's elements to writable receiver |
| `Items.PushInPlace(Value: T)` | Append one element to writable receiver |
| `Items.PopInPlace(): T` | Remove and return last element; empty arrays raise FP5003 |
| `Items.MapInPlace(F: function(X: T): T)` | Transform writable receiver without changing element type |
| `Items.FlatMapInPlace(F: function(X: T): array of T)` | Map and flatten writable receiver without changing element type |
| `Items.Join(Delim: string): string` | Join the receiver's strings with the delimiter |
| `array.Fill(Value: T; Count: integer): array of T` | Construct an array of repeated values |

`Length()` and `IsEmpty()` use the same canonical names for strings, arrays, and dictionaries.

See [collection mutation](../collection-mutation.md) for storage, evaluation,
callbacks and failure rules. InPlace transformations return no value except
PopInPlace. ArrayPopResult is an import-free generic record with Remaining:
array of T and Value: T; see [stack operations](mutating.md).


## Topics

| Topic | Description |
|-------|-------------|
| [Basics](basics.md) | `Length`, `Sort`, `SortBy`, `Reverse`, search, `Slice` |
| [Stack operations](mutating.md) | Value and InPlace `Push`/`Pop` |
| [Higher-order](higher-order.md) | `Map`, `Filter`, `Reduce`, `Find`, `Any`, `All` |
| [Combine and iterate](combine.md) | `Concat`, `FlatMap`, `Fill`, `ForEach` |

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Value algorithms | [`array.rs`](../../../../../crates/fpas-std/src/array.rs) |
| Key-selector callbacks | [`sorting.rs`](../../../../../crates/fpas-vm/src/vm/hosted/callbacks/sorting.rs) |
| `Push` / `Pop` | [`vm/mod.rs`](../../../../../crates/fpas-vm/src/vm/mod.rs), [`lowering/calls.rs`](../../../../../crates/fpas-compiler/src/lowering/calls.rs) |
| Registration | [`std_registry/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Collections index](../../../std/collections/README.md)
- [Standard library index](../../../std/README.md)
