# First-class functions

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
  var R: integer := Apply(Double, 5); // 10
  var Op: function(X: integer): integer := Double;
  Console.WriteLn(Op(7)); // 14
end program;
```

Call sites pass a **named** function or procedure, a **closure expression**, a
**bound record method**, or a **variable** whose type is a function or procedure
type. Qualified routines work the same way: `Console.WriteLn(...)` after `uses Std.Console as Console;`.

```pascal
program Example;

uses Std.Console as Console;

type Counter = record
  Base: integer;

  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value;
  end function;
end record;

begin
  var C: Counter := record
    Base := 10;
  end record;
  var AddTen: function(Value: integer): integer := C.Add;
  Console.WriteLn(AddTen(5)); // 15
  Console.WriteLn(AddTen(7)); // 17
end program;
```

`C.Add` captures `C` by value once; calling `AddTen` supplies only the remaining
parameters. See [Record methods](../types/record-methods.md#bound-methods-as-values).

## See also

- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [Record methods](../types/record-methods.md)
- [`Std.Arrays`](../../std/collections/array/README.md) — `Map`, `Filter`, and other higher-order helpers
