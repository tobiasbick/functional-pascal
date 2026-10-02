# Receiver calls

A value can use `Receiver.Name(Arguments)` when a visible local function,
procedure, or callable value named `Name` accepts the receiver as its first
explicit parameter. The receiver becomes that argument; the remaining arguments
keep their written order. Ordinary calls remain available.

```pascal
program ReceiverCalls;

uses Std.Test as Test;

function Add(Value: integer; Amount: integer): integer;
begin
  return Value + Amount;
end function;

begin
  Test.AssertEquals(42, (20).Add(22));
end program;
```

The receiver and every explicit argument are evaluated once, from left to right.
The syntax works after a designator, a parenthesized value, or another call. Each
step uses the previous result's static type. A bound method already has its
`Self` parameter bound; a receiver fills its first remaining parameter.

## Lookup

For a record receiver, a declared member of the same name takes priority.
Instance methods keep their existing behavior. Fields, properties, events,
private members, and static members do not fall through to a free routine if the
member call is invalid. A callable field or property takes only the arguments
written after its name; the record is not inserted as an argument.

Otherwise, the nearest lexical binding wins, even when its first parameter is
incompatible. Imports expose only their declared alias. Imported functions must
be called through that alias, for example `Arrays.Length(Items)`. They are not
candidates for `Items.Length()`.

After a callable is selected, argument count, types, generic inference,
constraints, and return type are checked as for an ordinary call. A callable with
no explicit parameters cannot be selected by a receiver.

## Procedures, mutation, and tasks

A procedure can end a call chain used as a statement. It cannot feed a later step
because it produces no value. An ordinary `mutable` parameter keeps its usual
binding semantics; it does not require a mutable receiver variable.

The imported `Std.Arrays.Push` and `Pop` intrinsics require a simple mutable array
variable as their first argument: `Arrays.Push(Items, Value)` and
`Arrays.Pop(Items)`. Indexed or field expressions and returned arrays are invalid
for those intrinsics.

`go Receiver.Name(Arguments)` spawns an eligible local call. Imported calls use
`go Alias.Name(Arguments)`. The callable and receiver are evaluated before the
explicit arguments.

## See also

- [Postfix chaining](postfix-chaining.md)
- [Record methods](../types/record-methods.md)
- [Imports](../../program-structure/units.md)
- [`Std.Arrays`](../../std/collections/array/README.md)
