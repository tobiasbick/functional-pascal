# Basics

## `Items.Length(): integer`

Number of elements in `A`.

```pascal
const A: array of integer := [1, 2, 3];
WriteLn(A.Length());
```

---

## `Items.Sort(): array of T`

Returns a **new** sorted array. **`A` is not modified.**

```pascal
const A: array of integer := [3, 1, 2];
const B: array of integer := A.Sort();
WriteLn(B.IndexOf(2));
```

---

## `Items.Reverse(): array of T`

Returns a **new** array with elements in reverse order. **`A` is not modified.**

```pascal
const A: array of integer := [1, 2, 3];
const R: array of integer := A.Reverse();
WriteLn(R.Length());
```

---

## `Items.Contains(Value: T): boolean`

`true` if some element equals `Value`.

```pascal
const A: array of integer := [1, 2, 3];
WriteLn(A.Contains(2));
WriteLn(A.Contains(99));
```

---

## `Items.IndexOf(Value: T): integer`

First index where `A[i] = Value`, or **`-1`**.

```pascal
WriteLn([10, 20, 30].IndexOf(20));
```

---

## `Items.Slice(Start: integer; Len: integer): array of T`

Copies `Len` elements starting at `Start`. **Runtime error** if the range is out of bounds.

```pascal
const A: array of integer := [10, 20, 30, 40];
const C: array of integer := A.Slice(1, 2);
WriteLn(C.Length());
```

## See also

- [Array overview](README.md)
- [Mutating](mutating.md)
- [Collections index](../../../std/README.md)
