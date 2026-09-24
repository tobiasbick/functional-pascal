# Receiver calls

Any value can use `Receiver.Name(Arguments)` when a visible function, procedure,
or callable value named `Name` accepts the receiver as its first explicit
parameter. The receiver becomes that argument; the other arguments retain
their written order. Ordinary calls remain available.

```pascal
uses Std.Arrays, Std.Test;

var Values: array of integer := [1, 2, 3, 4];
var Total: integer := Values.Filter(IsEven).Map(Double).Reduce(0, Sum);
AssertEquals(12, Total)
```

The expression means `Std.Arrays.Reduce(Std.Arrays.Map(Std.Arrays.Filter(
Values, IsEven), Double), 0, Sum)`. Each receiver and explicit argument is
evaluated once, from left to right. Every step uses the previous result's
static type, so a chain may change from an array to a scalar, `Option`, or
`Result`. No conversion, unwrap, retry, or lazy evaluation is implied.

The syntax works after a designator (`Values.Map(F)`), a parenthesized value
(`(2).Double()`), or another call (`MakeValues().Map(F)`). A function- or
procedure-typed local variable or parameter can be selected in the same way.
A bound method value already has its `Self` parameter bound; a receiver fills
its first remaining parameter.

## Lookup

For a record receiver, a declared member of the same name takes priority.
Instance methods keep their existing behavior. Fields, properties, events,
private members, and static members do not fall through to a free routine if
the member call is invalid. A callable field or property takes only the
arguments written after its name; the record is not inserted as an argument.

Otherwise, the nearest lexical binding wins. A local function, procedure, or
callable value shadows an imported routine even when its first parameter is
incompatible. Without a lexical binding, public symbols from the units in
`uses` are filtered by whether their first explicit parameter accepts the
receiver type. This can disambiguate imported short names:

```pascal
uses Std.Arrays, Std.Dictionaries, Std.Str;

var A: array of integer := [1];
var D: dict of string to integer := ['a': 2];
var Count: integer := A.Length() + D.Length() + ('hi').Length();
```

If several imported callables still match, use a qualified ordinary call.
Trailing arguments and the expected result type do not choose between them.
After one callable is selected, its argument count, types, generic inference,
constraints, and return type are checked just as for an ordinary call. A
callable with no explicit parameters cannot be selected by a receiver.

## Procedures and mutation

A procedure can end a call chain used as a statement. It cannot feed a later
step because it produces no value. An ordinary `mutable` parameter keeps its
usual binding semantics; it does not require a mutable receiver variable.

`Std.Arrays.Push` and `Pop` are stricter: their receiver must be a simple
mutable array variable. `Items.Push(Value)` and `Items.Pop()` work when
`Items` is such a variable. `(Items).Push(Value)`, indexed or field receivers,
and returned arrays are invalid for those intrinsics.

`go Receiver.Name(Arguments)` also spawns an eligible call. The receiver is
evaluated before its explicit arguments.

## See also

- [Postfix chaining](postfix-chaining.md)
- [Record methods](../types/record-methods.md)
- [Imports](../../program-structure/units.md)
- [`Std.Arrays`](../../std/collections/array/README.md)
