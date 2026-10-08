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

Parameters are read-only bindings unless they are declared with `var`. Copy a
read-only parameter into a local `var` when its value, fields, or elements need
to change. Changes to that copy do not change the caller's binding:

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

To change the caller's variable, declare a [`var` parameter](var-parameters.md)
and mark the argument: `Increase(var Counter)`.
For a named call, write `Increase(Value := var Counter)`.

## Named arguments

A call can pass every argument by name with `Name := Value`. Names are the
declared parameter names and are matched without regard to case. The order of
named arguments is free:

```pascal
procedure CopyFile(Source: string; Destination: string; Overwrite: boolean);
begin
end procedure;

begin
  CopyFile(Source := InputPath, Destination := BackupPath, Overwrite := false);
  CopyFile(Overwrite := true, Source := InputPath, Destination := BackupPath);
end.
```

Arguments are evaluated in written order, even when the names reorder the
parameters. A call is either fully positional or fully named; mixing both forms
is rejected (FP2016). Parameters have no default values, so a named call passes
every parameter exactly once. Unknown, duplicated and missing names are reported
with the declared parameter names (FP3025). Because parameter names are part of
the call syntax, renaming a public parameter changes the routine's API.

A [`var` parameter](var-parameters.md) uses `Name := var Designator` in a named
call. Read-only parameters still use `Name := Value`. The `var` marker is required
only for `var` parameters (FP3027), and the same writable-storage, exact-type,
aliasing, and lifetime checks apply as in positional calls. Reordering labels
does not change the evaluation order of values or reference roots and indices.

Named arguments apply to calls of declared functions and procedures, including
imported and `Std.*` routines with declared signatures, record methods
(`Point.Moved(Dx := 1, Dy := 2)`; `Self` is implicit), static record routines,
`go` calls of such routines, and enum variant constructors, whose field names
act as parameter names (`Shape.Rectangle(Width := 10.0, Height := 20.0)`; see
[Enums](../types/enums.md)). Fixed-signature native type operations and the
`string.Chr` / `array.Fill` factories also support named explicit arguments;
the implicit receiver cannot be named. The following forms take positional arguments only
and reject names (FP3026):

- function values: closures, callable bindings and parameters, and callable
  record fields, because function types do not carry parameter names;
- `Ok(…)`, `Error(…)`, and `Some(…)`, and enum patterns in `case` labels;
- variadic routines such as `WriteLn` and `Format`, and polymorphic
  standard-library operations such as `Abs`.

```pascal
const F: function(Value: integer): integer := Double;
const Answer: integer := F(3);          // valid
const Bad: integer := F(Value := 3);    // FP3026: F is a function value
```

## See also

- [Declarations](declarations.md)
