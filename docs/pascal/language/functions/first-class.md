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

## Optional handlers

A callback can be an ordinary record field of type `Option of HandlerType`.
Its `None` default represents the absence of a handler; `Some(...)` stores a
named routine, closure, or bound method. Use an `is` test when the callback
should run only when present:

```pascal
program OptionalHandlers;

uses Std.Console;

type ClickHandler = procedure(Sender: integer);
type Button = record
  Id: integer;
  OnClick: Option of ClickHandler := None;

  procedure Click(Self: Button);
  begin
    if Self.OnClick is Some(const Handler) then
      Handler(Self.Id);
    end if;
  end procedure;
end record;

procedure HandleClick(Sender: integer);
begin
  WriteLn(Sender);
end procedure;

begin
  var B: Button := Button(Id := 1);
  B.OnClick := Some(HandleClick);
  B.Click(); // Prints 1.
  B.OnClick := None;
  B.Click(); // No handler, so no call.
end.
```

Ordinary field rules apply: setting or clearing a handler requires a mutable
record binding; a `const` record can read the field and call an installed
handler. Construction and record updates can supply the field, and copies
retain their own handler values. Unit fields are private by default; `public`
exposes a field for ordinary reads, assignments, and calls from consumers.
The `None` default is preserved in imported records and transparent aliases.

When absence is an error, select the callable with `Value.OnClick.Unwrap()`
and call the resulting value. `Unwrap` panics for `None`; it does not supply a
default result. A full `case` may instead handle both `Some` and `None`
explicitly. Calls retain the normal callable parameter and capture rules.

## See also

- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [Record methods](../types/record-methods.md)
- [`Array operations`](../types/array/README.md) — `Map`, `Filter`, and other higher-order helpers
