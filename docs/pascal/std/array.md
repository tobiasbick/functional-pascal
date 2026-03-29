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

**Maintenance (implementers only):** align with [`std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs), [`array.rs`](../../../crates/fpas-std/src/array.rs), [`vm/`](../../../crates/fpas-vm/src/vm/mod.rs) and [`compiler.rs`](../../../crates/fpas-compiler/src/compiler.rs) for `Push`/`Pop`, plus [`intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs) / [`intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs).

---

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

---

## `function Length(A: array of T): integer`

Number of elements in `A`.

```pascal
var A: array of integer := [1, 2, 3];
WriteLn(Length(A))
```

---

## `function Sort(A: array of T): array of T`

Returns a **new** sorted array. **`A` is not modified.**

```pascal
var A: array of integer := [3, 1, 2];
var B: array of integer := Sort(A);
WriteLn(IndexOf(B, 2))
```

---

## `function Reverse(A: array of T): array of T`

Returns a **new** array with elements in reverse order. **`A` is not modified.**

```pascal
var A: array of integer := [1, 2, 3];
var R: array of integer := Reverse(A);
WriteLn(Length(R))
```

---

## `function Contains(A: array of T; Value: T): boolean`

`true` if some element equals `Value`.

```pascal
var A: array of integer := [1, 2, 3];
WriteLn(Contains(A, 2));
WriteLn(Contains(A, 99))
```

---

## `function IndexOf(A: array of T; Value: T): integer`

First index where `A[i] = Value`, or **`-1`**.

```pascal
WriteLn(IndexOf([10, 20, 30], 20))
```

---

## `function Slice(A: array of T; Start: integer; Len: integer): array of T`

Copies `Len` elements starting at `Start`. **Runtime error** if the range is out of bounds.

```pascal
var A: array of integer := [10, 20, 30, 40];
var C: array of integer := Slice(A, 1, 2);
WriteLn(Length(C))
```

---

## `procedure Push(mutable A: array of T; Value: T)`

Appends `Value` to the end of **`A`** (mutates `A`).

```pascal
mutable var A: array of integer := [1, 2];
Push(A, 3);
WriteLn(Length(A))
```

---

## `function Pop(mutable A: array of T): T`

Removes the **last** element and returns it. **`A` becomes shorter.** **Runtime error** if `A` is empty.

```pascal
mutable var A: array of integer := [1, 2, 3];
var Last: integer := Pop(A);
WriteLn(Last);
WriteLn(Length(A))
```

---

## `function Map(A: array of T; F: function(X: T): U): array of U`

Returns a new array where each element is the result of calling `F` on the corresponding element of `A`.

```pascal
var Nums: array of integer := [1, 2, 3];
var Doubled: array of integer := Map(Nums,
  function(X: integer): integer begin return X * 2 end);
```

---

## `function Filter(A: array of T; F: function(X: T): boolean): array of T`

Returns a new array containing only elements for which `F` returns `true`.

```pascal
var Nums: array of integer := [1, 2, 3, 4, 5];
var Evens: array of integer := Filter(Nums,
  function(X: integer): boolean begin return X mod 2 = 0 end);
```

---

## `function Reduce(A: array of T; Init: U; F: function(Acc: U; V: T): U): U`

Folds elements left-to-right, starting from `Init`.

```pascal
var Nums: array of integer := [1, 2, 3, 4, 5];
var Sum: integer := Reduce(Nums, 0,
  function(Acc: integer; V: integer): integer begin return Acc + V end);
```

---

## `function Find(A: array of T; F: function(X: T): boolean): Option of T`

Returns the **first** element for which `F` returns `true`, wrapped in `Some`. Returns `None` if no element matches. Requires `uses Std.Option` to work with the result.

```pascal
var Nums: array of integer := [1, 2, 3, 4, 5];
var First: Option of integer := Find(Nums,
  function(X: integer): boolean begin return X > 3 end);
{ Some(4) }
```

---

## `function FindIndex(A: array of T; F: function(X: T): boolean): integer`

Returns the **index** of the first element for which `F` returns `true`, or **`-1`** if none matches.

```pascal
var Idx: integer := FindIndex([10, 20, 30],
  function(X: integer): boolean begin return X > 15 end);
WriteLn(Idx)  { 1 }
```

---

## `function Any(A: array of T; F: function(X: T): boolean): boolean`

Returns `true` if **at least one** element satisfies `F`.

```pascal
var HasNeg: boolean := Any([1, -2, 3],
  function(X: integer): boolean begin return X < 0 end);
WriteLn(HasNeg)  { true }
```

---

## `function All(A: array of T; F: function(X: T): boolean): boolean`

Returns `true` if **every** element satisfies `F`.

```pascal
var AllPos: boolean := All([1, 2, 3],
  function(X: integer): boolean begin return X > 0 end);
WriteLn(AllPos)  { true }
```

---

## `function Concat(A: array of T; B: array of T): array of T`

Returns a **new** array containing all elements of `A` followed by all elements of `B`.

```pascal
var C: array of integer := Concat([1, 2], [3, 4]);
WriteLn(Length(C))  { 4 }
```

---

## `function FlatMap(A: array of T; F: function(X: T): array of U): array of U`

Applies `F` to each element (producing an array), then flattens all results into a single array.

```pascal
var Result: array of integer := FlatMap([1, 2, 3],
  function(X: integer): array of integer begin return [X, X * 10] end);
{ [1, 10, 2, 20, 3, 30] }
```

---

## `function Fill(Value: T; Count: integer): array of T`

Creates a new array containing `Count` copies of `Value`.

```pascal
var Zeros: array of integer := Fill(0, 5);
WriteLn(Length(Zeros))  { 5 }
```

---

## `procedure ForEach(A: array of T; F: procedure(X: T))`

Calls `F` for each element in `A`. Does not return a value.

```pascal
ForEach([1, 2, 3],
  procedure(X: integer) begin WriteLn(X) end);
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Pure helpers | [`array.rs`](../../../crates/fpas-std/src/array.rs) |
| `Push` / `Pop` | [`vm.rs`](../../../crates/fpas-vm/src/vm.rs), [`compiler.rs`](../../../crates/fpas-compiler/src/compiler.rs) |
| Registration | [`std_registry.rs`](../../../crates/fpas-sema/src/std_registry.rs) |

[← Standard library index](README.md)
