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
  var R: integer := Apply(Double, 5);  { 10 }
  var Op: function(X: integer): integer := Double;
  WriteLn(Op(7));                        { 14 }
end.
```

Call sites pass a **named** function or procedure, or a **variable** whose type is a function or procedure type. Qualified routines work the same way: `Std.Console.WriteLn(...)`.

## See also

- [Function types](function-types.md)
- [`Std.Array`](../../std/collections/array/README.md) — `Map`, `Filter`, and other higher-order helpers
