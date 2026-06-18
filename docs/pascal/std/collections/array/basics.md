# Basics

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

## See also

- [Array overview](README.md)
- [Mutating](mutating.md)
- [Collections index](../../README.md)
