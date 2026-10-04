# Mutating

## `procedure Push(var A: array of (T); Value: T)`

Appends `Value` to caller storage `A`. The first argument must use an explicit
`var` marker and select an array in a mutable binding or a forwarded `var`
parameter. Stored fields and array/dictionary elements are eligible targets.

```pascal
uses Std.Arrays as Arrays;
uses Std.Console as Console;

 var A: array of (integer) := [1, 2];
Arrays.Push(var A, 3);
Arrays.Push(var A, 4);
Console.WriteLn(Arrays.Length(A));
```

## `pure function Pop(var A: array of (T)): T`

Removes the last element, updates caller storage and returns the removed value.
An empty array produces a runtime bounds error at the source call.

```pascal
uses Std.Arrays as Arrays;
uses Std.Console as Console;

 var A: array of (integer) := [1, 2, 3];
const Last: integer := Arrays.Pop(var A);
const Next: integer := Arrays.Pop(var A);
Console.WriteLn(Last);
Console.WriteLn(Arrays.Length(A));
```

Both operations reserve the storage root before evaluating later arguments.
An ordinary value argument may read a snapshot of that reserved root; another
write or caller-storage reservation conflicts. Selected indices and keys are
evaluated once. Existing value copies remain independent through copy-on-write.
The callee activates the reference and releases it on normal return or failure.
See [var parameters](../../../language/functions/var-parameters.md).

## Implementation (contributors)

| Concern | Owner |
| --- | --- |
| Writable storage and argument checking | `fpas-sema` array builtins and shared var checker |
| Call reservation and concrete mutation entries | `fpas-compiler` call and closure lowering |
| Reference activation, array update and cleanup | `fpas-vm` calls and register operations |

## See also

- [Array overview](README.md)
- [Higher-order](higher-order.md)
- [Collections index](../README.md)
- [Standard library index](../../README.md)
