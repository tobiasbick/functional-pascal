# Higher-order

## `function Map(A: array of T; F: function(X: T): U): array of U`

Returns a new array where each element is the result of calling `F` on the corresponding element of `A`.

```pascal
function Double(X: integer): integer;
begin
  return X * 2;
end function;

var Nums: array of integer := [1, 2, 3];
var Doubled: array of integer := Map(Nums, Double);

```

---

## `function Filter(A: array of T; F: function(X: T): boolean): array of T`

Returns a new array containing only elements for which `F` returns `true`.

```pascal
function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0;
end function;

var Nums: array of integer := [1, 2, 3, 4, 5];
var Evens: array of integer := Filter(Nums, IsEven);

```

---

## `function Reduce(A: array of T; Init: U; F: function(Acc: U; V: T): U): U`

Folds elements left-to-right, starting from `Init`.

```pascal
function Sum(Acc: integer; V: integer): integer;
begin
  return Acc + V;
end function;

var Nums: array of integer := [1, 2, 3, 4, 5];
var Total: integer := Reduce(Nums, 0, Sum);

```

---

## `function Find(A: array of T; F: function(X: T): boolean): Option of T`

Returns the **first** element for which `F` returns `true`, wrapped in `Some`. Returns `None` if no element matches. Requires `uses Std.Options as Options;` to work with the result.

```pascal
function IsAboveThree(X: integer): boolean;
begin
  return X > 3;
end function;

var Nums: array of integer := [1, 2, 3, 4, 5];
var First: option of integer := Find(Nums, IsAboveThree);

```

---

## `function FindIndex(A: array of T; F: function(X: T): boolean): integer`

Returns the **index** of the first element for which `F` returns `true`, or **`-1`** if none matches.

```pascal
uses Std.Console as Console;

function IsAboveFifteen(X: integer): boolean;
begin
  return X > 15;
end function;

  var Idx: integer := FindIndex([10, 20, 30], IsAboveFifteen);
  Console.WriteLn(Idx); // 1
```

---

## `function Any(A: array of T; F: function(X: T): boolean): boolean`

Returns `true` if **at least one** element satisfies `F`.

```pascal
uses Std.Console as Console;

function IsNegative(X: integer): boolean;
begin
  return X < 0;
end function;

  var HasNeg: boolean := Any([1, -2, 3], IsNegative);
  Console.WriteLn(HasNeg); // true
```

---

## `function All(A: array of T; F: function(X: T): boolean): boolean`

Returns `true` if **every** element satisfies `F`.

```pascal
uses Std.Console as Console;

function IsPositive(X: integer): boolean;
begin
  return X > 0;
end function;

  var AllPos: boolean := All([1, 2, 3], IsPositive);
  Console.WriteLn(AllPos); // true
```

## See also

- [Array overview](README.md)
- [Combine and iterate](combine.md)
- [`Std.Options`](../../result/option.md)
