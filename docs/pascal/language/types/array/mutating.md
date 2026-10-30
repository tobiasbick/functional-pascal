# Array stack operations

Push appends at the end; Pop removes the last element. Value forms preserve the
original array. InPlace forms update writable caller storage.

## Value operations

Items.Push(Value: T): array of T returns the appended array. Items.Pop():
ArrayPopResult of T returns a named generic record with public fields:

| Field | Type | Meaning |
| --- | --- | --- |
| Remaining | array of T | New array without the last element. |
| Value | T | Removed last element. |

ArrayPopResult needs no import and supports annotations and generic signatures.
Its fields follow ordinary record access and construction rules.

```pascal id=example-1 fragment context=body
const Items := [10, 20, 30];
const Extended := Items.Push(Value := 40);
const Popped: ArrayPopResult of integer := Items.Pop();
// Extended is [10, 20, 30, 40].
// Popped.Remaining is [10, 20]; Popped.Value is 30.
// Items remains [10, 20, 30].
```

## Mutating operations

Items.PushInPlace(Value: T) updates the receiver and returns no value.
Items.PopInPlace(): T updates it and returns only the removed element.

```pascal id=example-2 fragment context=body
var Items := [10, 20];
Items.PushInPlace(30);
const Last := Items.PopInPlace();
// Last is 30; Items is [10, 20].
```

Both Pop forms retain FP5003 on empty arrays. Failed empty PopInPlace does not
alter storage. Check IsEmpty first when emptiness is expected.

Positional and fully named arguments are supported. Value is read-only;
Items.PushInPlace(var Value) is rejected. Bind the value operations as
`Items.Push` or `Items.Pop` when passing callbacks. InPlace operations require
writable source storage; use an ordinary routine with an explicit `var` parameter
when a mutating callback is needed. No separate free-routine API is provided.

Storage, path evaluation, aliases, callback/failure rules and go restrictions
are specified in [collection mutation](../collection-mutation.md). Direct local
PushInPlace/PopInPlace can reuse unique storage, but allocation is not guaranteed
to be avoided. Other value copies always remain unchanged.
