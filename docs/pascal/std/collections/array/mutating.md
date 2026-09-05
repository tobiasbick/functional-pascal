# Mutating

## `procedure Push(mutable A: array of T; Value: T)`

Appends `Value` to the end of **`A`** (mutates `A`).

```pascal
mutable var A: array of integer := [1, 2];
Push(A, 3);
WriteLn(Length(A))
```

---

## `function Pop(mutable A: array of T): T`

Removes the **last** element and returns it. **`A` becomes shorter.** **Runtime error** if `A` is empty.

```pascal
mutable var A: array of integer := [1, 2, 3];
var Last: integer := Pop(A);
WriteLn(Last);
WriteLn(Length(A))
```

For a directly stored local array, `Pop` reuses uniquely owned storage. If another
value shares that array, copy-on-write preserves the other value. Global and
captured variables retain the general read-and-assign implementation.

## See also

- [Array overview](README.md)
- [Higher-order](higher-order.md)
- [Collections index](../../README.md)
