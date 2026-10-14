# Higher-order

## `Items.Map(F: function(X: T): U): array of U`

Returns a new array where each element is the result of calling `F` on the corresponding element of `A`.

```pascal
function Double(X: integer): integer;
begin
  return X * 2;
end function;

const Nums: array of integer := [1, 2, 3];
const Doubled: array of integer := Nums.Map(Double);
```

---

## `Items.Filter(F: function(X: T): boolean): array of T`

Returns a new array containing only elements for which `F` returns `true`.

```pascal
function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0;
end function;

const Nums: array of integer := [1, 2, 3, 4, 5];
const Evens: array of integer := Nums.Filter(IsEven);
```

---

## `Items.Reduce(Init: U; F: function(Acc: U; V: T): U): U`

Folds elements left-to-right, starting from `Init`.

```pascal
function Sum(Acc: integer; V: integer): integer;
begin
  return Acc + V;
end function;

const Nums: array of integer := [1, 2, 3, 4, 5];
const Total: integer := Nums.Reduce(0, Sum);
```

---

## `Items.Find(F: function(X: T): boolean): Option of T`

Returns the **first** element for which `F` returns `true`, wrapped in `Some`. Returns `None` if no element matches. The `Option` result needs no import.

```pascal
function IsAboveThree(X: integer): boolean;
begin
  return X > 3;
end function;

const Nums: array of integer := [1, 2, 3, 4, 5];
const First: option of integer := Nums.Find(IsAboveThree);
// Some(4)
```

---

## `Items.FindIndex(F: function(X: T): boolean): Option of integer`

Returns `Some(Index)` for the first zero-based index whose element makes F return
`true`, or `None` if none matches. Visits elements in order and stops at the first
match. Empty arrays return `None` without invoking F; callback failures propagate.

```pascal
function IsAboveFifteen(X: integer): boolean;
begin
  return X > 15;
end function;

const Idx: option of integer := [10, 20, 30].FindIndex(IsAboveFifteen);
case Idx of
  when Some(const Index):
    Console.WriteLn(Index); // 1
  when None:
    Console.WriteLn('absent');
end case;
```

---

## `Items.Any(F: function(X: T): boolean): boolean`

Returns `true` if **at least one** element satisfies `F`.

```pascal
function IsNegative(X: integer): boolean;
begin
  return X < 0;
end function;

const HasNeg: boolean := [1, -2, 3].Any(IsNegative);
Console.WriteLn(HasNeg); // true
```

---

## `Items.All(F: function(X: T): boolean): boolean`

Returns `true` if **every** element satisfies `F`.

```pascal
function IsPositive(X: integer): boolean;
begin
  return X > 0;
end function;

const AllPos: boolean := [1, 2, 3].All(IsPositive);
Console.WriteLn(AllPos); // true
```

## See also

- [Array overview](README.md)
- [Combine and iterate](combine.md)
- [`Option operations`](../option-operations.md)
