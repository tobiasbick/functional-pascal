# First-class functions

Functions can be assigned to variables and passed as arguments:

```pascal
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
  WriteLn(Op(7)); // 14
end.
```

Call sites pass a **named** function or procedure, a **closure expression**, a
**bound record method**, or a **variable** whose type is a function or procedure
type. Qualified routines work the same way: `Std.Console.WriteLn(...)`.

Array elements and dictionary entries with callable types are also called
through their designators. The indexed callable is selected once before its
arguments are evaluated:

```pascal
const Operations: array of function(X: integer): integer := [Double];
const ByName: dict of string to function(X: integer): integer := ['double': Double];
const First: integer := Operations[0](7); // 14
const Named: integer := ByName['double'](7); // 14
```

These calls use positional arguments, like other first-class callable values.
Procedure elements may be called as statements. Their `var` parameter modes
and the restrictions on [task calls](../concurrency/go.md) remain the same.

```pascal
type
  Counter = record
    Base: integer;

    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value;
    end function;
  end record;

begin
  const C: Counter := Counter(
    Base := 10
  );
  const AddTen: function(Value: integer): integer := C.Add;
  WriteLn(AddTen(5)); // 15
  WriteLn(Apply(AddTen, 7)); // 17
end.
```

`C.Add` captures `C` by value once; calling `AddTen` supplies only the remaining
parameters. See [Record methods](../types/record-methods.md#bound-methods-as-values).

## See also

- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [Record methods](../types/record-methods.md)
- [`Array operations`](../types/array/README.md) — `Map`, `Filter`, and other higher-order helpers
