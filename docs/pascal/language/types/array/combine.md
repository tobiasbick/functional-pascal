# Combine and iterate

## `Items.Concat(B: array of T): array of T`

Returns a **new** array containing all elements of `A` followed by all elements of `B`.

```pascal id=example-1 fragment context=body-console
const C: array of integer := [1, 2].Concat([3, 4]);
Console.WriteLn(C.Length()); // 4
```

---

## `Items.FlatMap(F: function(X: T): array of U): array of U`

Applies `F` to each element (producing an array), then flattens all results into a single array.

```pascal id=example-2 fragment context=declarations
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

```pascal id=example-3 fragment context=body-console
const Zeros: array of integer := array.Fill(0, 5);
Console.WriteLn(Zeros.Length()); // 5
```

`Count` must be non-negative and at most **1_000_000**. Larger counts raise a runtime error instead of allocating unbounded memory.

---

## `Items.ForEach(F: procedure(X: T))`

Calls `F` for each element in `A`. Does not return a value.

```pascal id=example-4 fragment context=header-console
procedure PrintValue(X: integer);
begin
  Console.WriteLn(X);
end procedure;
begin

[1, 2, 3].ForEach(PrintValue);

end.
```

## See also

- [Array overview](README.md)
- [Higher-order](higher-order.md)
- [Collections index](../../../std/README.md)
