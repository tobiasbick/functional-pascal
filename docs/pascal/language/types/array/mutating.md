# Mutating

## `Items.Push(Value: T)`

Appends `Value` to the end of **`A`** (mutates `A`).

```pascal
var A: array of integer := [1, 2];
A.Push(3);
A.Push(4);
WriteLn(A.Length());
```

---

## `Items.Pop(): T`

Removes the **last** element and returns it. **`A` becomes shorter.** **Runtime error** if `A` is empty.

```pascal
var A: array of integer := [1, 2, 3];
const Last: integer := A.Pop();
const Next: integer := A.Pop();
WriteLn(Last);
WriteLn(A.Length());
```

For a directly stored local array, `Pop` reuses uniquely owned storage. If another
value shares that array, copy-on-write preserves the other value. Global, captured, field, element, and forwarded receivers use reference reads
and writes.
## Writable receivers

No import is required for these operations. Dot calls select the
receiver implicitly: `Items.Push(Value)` and `Items.Pop()` have no receiver
`var` marker or additional parentheses. Both require the same writable storage
as a [`var` argument](../../functions/var-parameters.md):

- a local, program, or unit `var` array, including an imported public variable;
- an array field of a writable record, `State.Items.Push(Value)`;
- an array element of writable storage, `Rows[Index].Pop()`;
- a forwarded `var` parameter, including its fields or elements.

Constants, read-only parameters, loop variables, properties, dictionary entries,
and computed or returned arrays are rejected (FP3028). Parenthesized values are
expressions rather than writable receiver designators.

```pascal
procedure Append(var Items: array of integer; Value: integer);
begin
  Items.Push(Value);
end procedure;

var Items: array of integer := [1, 2];
Append(var Items, 3);
const Last: integer := Items.Pop();
```

The receiver's root and indices are determined once before the explicit
arguments, in written order. Mutation then reads and writes that storage through
the same reference used by `var` parameters. Changes performed while evaluating
an argument remain visible; the receiver is not an earlier array snapshot.
Copy-on-write preserves other array or record values sharing its storage.

Explicit arguments keep their declared modes: `Value` is read-only, so
`Items.Push(var Value)` is invalid (FP3027). `Items.Push(Value := Value)` is also accepted. The receiver cannot be named;
there are no parallel free `Push` or `Pop` routines.

A writable receiver follows the shared aliasing and lifetime rules even though
it has no marker. Two writable arguments of one call cannot share a root
(FP3029); `Push`'s value argument may read the receiver, as in
`Items.Push(Items[0])`. A closure cannot capture a `var` parameter, and neither
`go Items.Push(Value)` nor `go Items.Pop()` can pass writable storage to another
task (FP3030). Call the operation on the current task instead.

Writes completed before a failure or early `try` exit remain visible. A failed
`Pop` on an empty array does not shorten it. `Push` is a procedure and can only
end a statement chain; `Pop` returns the removed element and that value can
continue a chain or initialize a `const` binding.

## See also

- [Array overview](README.md)
- [Higher-order](higher-order.md)
- [Collections index](../../../std/README.md)
