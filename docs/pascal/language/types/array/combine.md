# Combine and iterate

## `Items.Concat(B: array of T): array of T`

Returns a **new** array containing all elements of `A` followed by all elements of `B`.

```pascal
const C: array of integer := [1, 2].Concat([3, 4]);
WriteLn(C.Length());  // 4
```

---

## `Items.FlatMap(F: function(X: T): array of U): array of U`

Applies `F` to each element (producing an array), then flattens all results into a single array.

```pascal
function ExpandPair(X: integer): array of integer;
begin
  return [X, X * 10];
end function;

const Output: array of integer := [1, 2, 3].FlatMap(ExpandPair);
// [1, 10, 2, 20, 3, 30]
```

---

## `array.Fill(Value: T; Count: integer): array of T`

Creates a new array containing `Count` copies of `Value`.

```pascal
const Zeros: array of integer := array.Fill(0, 5);
WriteLn(Zeros.Length());  // 5
```

`Count` must be non-negative and at most **1_000_000**. Larger counts raise a runtime error instead of allocating unbounded memory.

---

## `Items.ForEach(F: procedure(X: T))`

Calls `F` for each element in `A`. Does not return a value.

```pascal
procedure PrintValue(X: integer);
begin
  WriteLn(X);
end procedure;

[1, 2, 3].ForEach(PrintValue);
```

## See also

- [Array overview](README.md)
- [Higher-order](higher-order.md)
- [Collections index](../../../std/README.md)
