# Higher-order

## `pure function Map(A: array of (T); F: pure function(X: T): U): array of (U)`

Returns a new array where each element is the result of calling `F` on the corresponding element of `A`.

```pascal
pure function Double(X: integer): integer;
begin
  return X * 2;
end function;

const Nums: array of (integer) := [1, 2, 3];
const Doubled: array of (integer) := Map(Nums, Double);

```

---

## `pure function Filter(A: array of (T); F: pure function(X: T): boolean): array of (T)`

Returns a new array containing only elements for which `F` returns `true`.

```pascal
pure function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0;
end function;

const Nums: array of (integer) := [1, 2, 3, 4, 5];
const Evens: array of (integer) := Filter(Nums, IsEven);

```

---

## `pure function Reduce(A: array of (T); Init: U; F: pure function(Acc: U; V: T): U): U`

Folds elements left-to-right, starting from `Init`.

```pascal
pure function Sum(Acc: integer; V: integer): integer;
begin
  return Acc + V;
end function;

const Nums: array of (integer) := [1, 2, 3, 4, 5];
const Total: integer := Reduce(Nums, 0, Sum);

```

---

## `pure function Find(A: array of (T); F: pure function(X: T): boolean): Option of (T)`

Returns the **first** element for which `F` returns `true`, wrapped in `Option.Some`. Returns `Option.None` if no element matches. Requires `uses Std.Options as Options;` to work with the result.

```pascal
pure function IsAboveThree(X: integer): boolean;
begin
  return X > 3;
end function;

const Nums: array of (integer) := [1, 2, 3, 4, 5];
const First: option of (integer) := Find(Nums, IsAboveThree);

```

---

## `pure function FindIndex(A: array of (T); F: pure function(X: T): boolean): integer`

Returns the **index** of the first element for which `F` returns `true`, or **`-1`** if none matches.

```pascal
uses Std.Console as Console;

pure function IsAboveFifteen(X: integer): boolean;
begin
  return X > 15;
end function;

  const Idx: integer := FindIndex([10, 20, 30], IsAboveFifteen);
  Console.WriteLn(Idx); // 1
```

---

## `pure function Any(A: array of (T); F: pure function(X: T): boolean): boolean`

Returns `true` if **at least one** element satisfies `F`.

```pascal
uses Std.Console as Console;

pure function IsNegative(X: integer): boolean;
begin
  return X < 0;
end function;

  const HasNeg: boolean := Any([1, -2, 3], IsNegative);
  Console.WriteLn(HasNeg); // true
```

---

## `pure function All(A: array of (T); F: pure function(X: T): boolean): boolean`

Returns `true` if **every** element satisfies `F`.

```pascal
uses Std.Console as Console;

pure function IsPositive(X: integer): boolean;
begin
  return X > 0;
end function;

  const AllPos: boolean := All([1, 2, 3], IsPositive);
  Console.WriteLn(AllPos); // true
```

## See also

- [Array overview](README.md)
- [Combine and iterate](combine.md)
- [`Std.Options`](../../result/option.md)
