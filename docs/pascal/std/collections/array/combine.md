# Combine and iterate

## `function Concat(A: array of (T); B: array of (T)): array of (T)`

Returns a **new** array containing all elements of `A` followed by all elements of `B`.

```pascal
uses Std.Console as Console;

var C: array of (integer) := Concat([1, 2], [3, 4]);
Console.WriteLn(Length(C)); // 4
```

---

## `function FlatMap(A: array of (T); F: function(X: T): array of (U)): array of (U)`

Applies `F` to each element (producing an array), then flattens all results into a single array.

```pascal
function ExpandPair(X: integer): array of (integer);
begin
  return [X, X * 10];
end function;

var Output: array of (integer) := FlatMap([1, 2, 3], ExpandPair);

```

---

## `function Fill(Value: T; Count: integer): array of (T)`

Creates a new array containing `Count` copies of `Value`.

```pascal
uses Std.Console as Console;

var Zeros: array of (integer) := Fill(0, 5);
Console.WriteLn(Length(Zeros)); // 5
```

`Count` must be non-negative and at most **1_000_000**. Larger counts raise a runtime error instead of allocating unbounded memory.

---

## `procedure ForEach(A: array of (T); F: procedure(X: T))`

Calls `F` for each element in `A`. Does not return a value.

```pascal
uses Std.Console as Console;

procedure PrintValue(X: integer);
begin
  Console.WriteLn(X);
end procedure;

  ForEach([1, 2, 3], PrintValue);
```

## See also

- [Array overview](README.md)
- [Higher-order](higher-order.md)
- [Collections index](../../README.md)
