# Basics

## `function Length(A: array of (T)): integer`

Number of elements in `A`.

```pascal
uses Std.Console as Console;

var A: array of (integer) := [1, 2, 3];
Console.WriteLn(Length(A));
```

---

## `function Sort(A: array of (T)): array of (T)`

Returns a **new** sorted array. **`A` is not modified.**

```pascal
uses Std.Console as Console;

var A: array of (integer) := [3, 1, 2];
var B: array of (integer) := Sort(A);
Console.WriteLn(IndexOf(B, 2));
```

---

## `function Reverse(A: array of (T)): array of (T)`

Returns a **new** array with elements in reverse order. **`A` is not modified.**

```pascal
uses Std.Console as Console;

var A: array of (integer) := [1, 2, 3];
var R: array of (integer) := Reverse(A);
Console.WriteLn(Length(R));
```

---

## `function Contains(A: array of (T); Value: T): boolean`

`true` if some element equals `Value`.

`Contains` and `IndexOf` use structural equality, including nested collections.
Resource, task or callable components are rejected; use `FindIndex` with an
explicit predicate for those types. NaN differs from itself, and positive and
negative zero compare equal.

```pascal
uses Std.Console as Console;

var A: array of (integer) := [1, 2, 3];
Console.WriteLn(Contains(A, 2));
Console.WriteLn(Contains(A, 99));
```

---

## `function IndexOf(A: array of (T); Value: T): integer`

First index where `A[i] = Value`, or **`-1`**.

```pascal
uses Std.Console as Console;

Console.WriteLn(IndexOf([10, 20, 30], 20));
```

---

## `function Slice(A: array of (T); Start: integer; Len: integer): array of (T)`

Copies `Len` elements starting at `Start`. **Runtime error** if the range is out of bounds.

```pascal
uses Std.Console as Console;

var A: array of (integer) := [10, 20, 30, 40];
var C: array of (integer) := Slice(A, 1, 2);
Console.WriteLn(Length(C));
```

## See also

- [Array overview](README.md)
- [Mutating](mutating.md)
- [Collections index](../../README.md)
