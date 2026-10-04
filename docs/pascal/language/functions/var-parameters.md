# Var parameters

A `var` parameter gives a synchronous routine writable caller storage. Both the
formal parameter and the actual argument require the marker:

```pascal
program CallerStorage;

procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;

begin
   var Count: integer := 1;
  Increase(var Count);
end program;
```

`Count` becomes `2`. Ordinary parameters are read-only value snapshots. To change
only a routine's local value, declare an explicit `var` copy in its body.
The obsolete `mutable Value: Type` formal parameter is rejected with that hint.

## Writable paths and types

A var argument names a mutable local or global, a forwarded var parameter, or a
stored field/array element/existing dictionary entry rooted in that storage.
Imported roots use their declared alias, which preserves the root's permissions. Immutable
bindings, value parameters, loop variables, temporaries, computed properties and
string elements cannot supply writable caller storage.

The selected storage type must match the formal type without a value conversion.
An integer root cannot satisfy `var Value: real`. Callable types preserve each
parameter's value/var mode, including generic routines, returned callable values,
callable fields and indexed callables.

## Evaluation and exclusive access

The callable target and arguments evaluate left to right. Each var root is
reserved when reached; each selected index is evaluated and checked once. Later
arguments may read snapshots of that root but cannot mutate it through an alias.
Two var arguments cannot share one storage root, even for different fields or
indices. Resolved duplicate roots are rejected before execution; aliases to the
same allocation are also checked at runtime.

On callee entry the reference becomes exclusive. Other aliases cannot read or
write its root until the call returns. Forwarding `Other(var Value)` reborrows
the same selected path; the parent's access resumes when that call finishes.
Writes take effect immediately. Return, failure and cancellation release the
authority without rolling back earlier writes. Conflicting alias access reports
F4026.

References are not ordinary values. They cannot be stored in records or
collections, returned, captured by anonymous or named nested routines, or sent
into a spawned task. A local value snapshot of a var parameter can be captured.
Independent value copies retain their nested copy-on-write isolation.

## See also

- [Parameters](parameters.md)
- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [Variables](../basics/variables.md)
