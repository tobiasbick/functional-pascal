# First-class functions

Functions can be assigned to variables and passed as arguments:

```pascal
function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value);
end;

function Double(X: integer): integer;
begin
  return X * 2;
end;

begin
  var R: integer := Apply(Double, 5);  // 10
  var Op: function(X: integer): integer := Double;
  WriteLn(Op(7));                      // 14
end.
```

Call sites pass a **named** function or procedure, a **closure expression**, a
**bound record method**, or a **variable** whose type is a function or procedure
type. Qualified routines work the same way: `Std.Console.WriteLn(...)`.

```pascal
type
  Counter = record
    Base: integer;

    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;

begin
  var C: Counter := record Base := 10; end;
  var AddTen: function(Value: integer): integer := C.Add;
  WriteLn(AddTen(5));           // 15
  WriteLn(Apply(AddTen, 7));    // 17
end.
```

`C.Add` captures `C` by value once; calling `AddTen` supplies only the remaining
parameters. See [Record methods](../types/record-methods.md#bound-methods-as-values).

## See also

- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [Record methods](../types/record-methods.md)
- [`Std.Array`](../../std/collections/array/README.md) — `Map`, `Filter`, and other higher-order helpers
