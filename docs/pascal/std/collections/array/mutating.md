# Mutating

## `procedure Push(A: array of T; Value: T)`

Appends `Value` to the end of **`A`** (mutates `A`).

```pascal
var A: array of integer := [1, 2];
Push(A, 3);
A.Push(4);
WriteLn(Length(A));
```

---

## `function Pop(A: array of T): T`

Removes the **last** element and returns it. **`A` becomes shorter.** **Runtime error** if `A` is empty.

```pascal
var A: array of integer := [1, 2, 3];
const Last: integer := Pop(A);
const Next: integer := A.Pop();
WriteLn(Last);
WriteLn(Length(A));
```

For a directly stored local array, `Pop` reuses uniquely owned storage. If another
value shares that array, copy-on-write preserves the other value. Global and
captured variables retain the general read-and-assign implementation.
In receiver-call syntax, the target must still be a simple mutable array
variable. Parenthesized names, fields, indexes, and returned arrays are not
accepted as receivers for `Push` or `Pop`.

## See also

- [Array overview](README.md)
- [Higher-order](higher-order.md)
- [Collections index](../../README.md)
