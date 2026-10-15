# Basics

## `Items.Length(): integer`

Number of elements in `A`.

```pascal
const A: array of integer := [1, 2, 3];
Console.WriteLn(A.Length());
```

---

## `Items.Sort(): array of T`

Returns a **new** sorted array. **`A` is not modified.**

Sorting is ascending and stable, using the same order as `<`. Equal values keep
their input order, including `0.0` and `-0.0`. Supported element types are integer,
real, Boolean, string, subranges, distinct scalar types, and enums without payload
fields. Real values follow the ordinary finite-real comparison rules.

The element type must satisfy `Comparable`. Unsupported types, including records,
payload-carrying enums, Option, Result, arrays, dictionaries, callables, tasks, and
channels, are rejected at compile time for both empty and populated arrays.
Equality support alone does not supply ordering. A generic array requires
`T: Comparable` or `T: Numeric`; `T: Printable` alone is insufficient. Aliases and
nominal element types are preserved. `SortInPlace()` applies the same ordering
checks and replaces writable storage without returning a result; see
[collection mutation](../collection-mutation.md).

An empty literal needs element-type evidence from its context or later native
arguments, for example `const Empty: array of integer := [].Sort();` or
`[].Sort().Push(1)`. That inferred input type must also satisfy `Comparable`,
even when a later `Map` changes the result's element type. A bare `[].Sort()`
without evidence is rejected.

```pascal
const A: array of integer := [3, 1, 2];
const B: array of integer := A.Sort();
Console.WriteLn(B.IndexOf(2));
```

---

## `Items.SortBy(F: function(X: T): K): array of T`

Returns a new array sorted ascending by the keys returned by `F`. The receiver
is preserved. `SortByInPlace(F)` applies the same ordering to writable storage
and returns no value. Both forms are stable: elements with equal keys keep their
input order. Keys use the same `<` ordering and finite-real rules as `Sort`.

Only the key type `K` must satisfy `Comparable`; elements can be records or any
other array element type. Unsupported keys fail checking even for empty arrays.
Generic keys require `K: Comparable` or `K: Numeric`. Use a named function or an
anonymous function; its single read-only parameter has the array's element type.
The named argument is `F`, consistent with other unary array callbacks.

The callback runs exactly once per element, from left to right over a snapshot
of the input, before sorting starts. Empty arrays invoke it zero times; singleton
arrays invoke it once. The snapshot and cached keys survive task suspension.
On callback failure no sorted result is committed. Other callback effects remain
visible, following [collection mutation](../collection-mutation.md).

```pascal
type Edit = record
  Offset: integer;
  Text: string;
end record;

const Edits := [Edit(Offset := 2, Text := 'later'), Edit(Offset := 0, Text := 'first')];
const Ascending := Edits.SortBy(
  function(Entry: Edit): integer begin
    return Entry.Offset;
  end function
);
var Descending := Edits;
Descending.SortByInPlace(
  function(Entry: Edit): integer begin
    return - Entry.Offset;
  end function
);
```

Negating nonnegative offsets produces descending offset order. Ordinary integer
overflow rules still apply when computing keys. Use parameterless `Sort()` for
arrays whose elements already have the required order.

---

## `Items.Reverse(): array of T`

Returns a **new** array with elements in reverse order. **`A` is not modified.**

```pascal
const A: array of integer := [1, 2, 3];
const R: array of integer := A.Reverse();
Console.WriteLn(R.Length());
```

---

## Membership with `in`

`Value in Items` is `true` if some element equals Value under the existing
equality and type-compatibility rules.

```pascal
const A: array of integer := [1, 2, 3];
Console.WriteLn(2 in A);
Console.WriteLn(99 in A);
```

---

## `Items.IndexOf(Value: T): Option of integer`

Returns `Some(Index)` for the first zero-based index where `Items[Index] = Value`,
or `None` if absent. An empty array returns `None`.

```pascal
case [10, 20, 30].IndexOf(20) of
  when Some(const Index):
    Console.WriteLn(Index); // 1
  when None:
    Console.WriteLn('absent');
end case;
```

---

## `Items.Slice(Start: integer; Len: integer): array of T`

Copies `Len` elements starting at `Start`. **Runtime error** if the range is out of bounds.

```pascal
const A: array of integer := [10, 20, 30, 40];
const C: array of integer := A.Slice(1, 2);
Console.WriteLn(C.Length());
```

## See also

- [Array overview](README.md)
- [Mutating](mutating.md)
- [Collections index](../../../std/README.md)
