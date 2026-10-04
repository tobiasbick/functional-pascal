# First-class functions and procedures

Functions can be assigned to variables and passed as arguments:

```pascal
program Example;

uses Std.Console as Console;

function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value);
end function;

function Double(X: integer): integer;
begin
  return X * 2;
end function;

begin
  const R: integer := Apply(Double, 5); // 10
  const Op: function(X: integer): integer := Double;
  Console.WriteLn(Op(7)); // 14
end program;
```

Functions and procedures are callable values. They can be stored, passed, returned
and copied. A procedure value can initialize a binding; calling that procedure
produces no value and cannot initialize a binding.

The invocation suffix `(Arguments)` accepts any expression with a function or
procedure type: a returned callable, an indexed value, a parenthesized callable,
an anonymous routine or a callable record field. The target is evaluated once
before the arguments, which are evaluated once from left to right. Only the
explicit arguments are passed; an ordinary callable field receives no implicit
record receiver. `try` in an argument preserves earlier evaluated values; an
error stops evaluation of later arguments and the callable body.

```pascal
program Composition;

type Handler = function(Value: integer): integer;

function MakeAdder(Base: integer): Handler;
begin
  return function(Value: integer): integer
  begin
    return Base + Value;
  end function;
end function;

begin
  const Answer: integer := MakeAdder(3)(5);
  const Callbacks: array of (Handler) := [MakeAdder(40)];
  const Other: integer := Callbacks[0](2);
  discard (Callbacks[0])(1);
end program;
```

Named and qualified routines retain ordinary call syntax, for example
`Console.WriteLn(...)` after `uses Std.Console as Console;`.

## Result consumption and discard

A function result must be used: store it, return it or pass it to another
expression. To intentionally ignore a value, write `discard Expression;`.
This also applies to the final function call in a postfix chain. `discard`
evaluates its operand exactly once and accepts ordinary values, including
Option/Result and callable values.

```pascal
discard MakeAdder(3)(5);
discard Option.Some(42);
```

A procedure call is an action statement, such as `Callbacks[0]();` for an
array of procedures. It cannot appear in a value position or as the operand
of `discard`. A task handle cannot be discarded, nor can a value whose type
contains task handles in a record, enum, collection, Option or Result. Wait
for the task explicitly, then use or discard its ordinary result.

## Capturing record values

An ordinary closure can retain a record snapshot and pass it explicitly to a
function:

```pascal
program RecordClosure;

uses Std.Console as Console;

type Counter = record
  Base: integer;
end record;

function CounterAdd(Receiver: Counter; Value: integer): integer;
begin
  return Receiver.Base + Value;
end function;

begin
  const C := Counter(Base := 10);
  const AddTen := function(Value: integer): integer
  begin
    return CounterAdd(C, Value);
  end function;
  Console.WriteLn(AddTen(5)); // 15
end program;
```

For an optional handler, store `Option of (procedure(...))` in a field,
match `Option.Some(const Handler)` or `Option.None`, and call the selected
handler with its explicit arguments.

## See also

- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [`Std.Arrays`](../../std/collections/array/README.md) — `Map`, `Filter`, and other higher-order helpers
