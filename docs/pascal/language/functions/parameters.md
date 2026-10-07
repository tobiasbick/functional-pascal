# Parameters

Parameters are separated by semicolons in declarations. Calls use commas. Each parameter requires a type annotation:

Comma-separated declarations such as `A: integer, B: integer` and grouped names
such as `A, B: integer` produce [FP2014](../../tools/diagnostics.md#parser), with
a hint showing `function Add(A: integer; B: integer): integer;`. The same
diagnostic applies to procedures, record methods, anonymous routines and
callable types. Commas inside a type such as `Result of integer, string`
are allowed.

```pascal
function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min;
  elsif Value > Max then
    return Max;
  else
    return Value;
  end if;
end function;

begin
  const R: integer := Clamp(150, 0, 100);  // 100
end.
```

## Read-only value parameters

Parameters are read-only bindings. Copy a parameter into a local `var` when
its value, fields, or elements need to change. Changes to that copy do not
change the caller's binding:

```pascal
function Increment(Value: integer): integer;
begin
  var LocalValue: integer := Value;
  LocalValue := LocalValue + 1;
  return LocalValue;
end function;
```

Arrays, dictionaries, and records retain value semantics when copied. Handles
such as channels still refer to their shared resource; a read-only binding
does not prevent operations on that resource. Closures capture a value
parameter by value, and a local `var` by shared mutable cell.

## See also

- [Declarations](declarations.md)
