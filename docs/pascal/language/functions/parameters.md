# Parameters

Parameters are separated by semicolons in declarations. Calls use commas. Each parameter requires a type annotation:

Comma-separated declarations such as `A: integer, B: integer` and grouped names
such as `A, B: integer` produce [FP2014](../../tools/diagnostics.md#parser), with
a hint showing `function Add(A: integer; B: integer): integer;`. The same
diagnostic applies to procedures, record methods, anonymous routines and
callable types. Commas inside a type such as `Result of (integer, string)`
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
  const R: integer := Clamp(150, 0, 100); // 100
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
the implicit receiver cannot be named. Callable bindings, parameters, fields,
closures and bound methods also accept names from their visible static signature.
Fixed-arity polymorphic standard operations use their published labels, for
example `Abs(N := -3)` and `Clamp(Hi := 10, V := 20, Lo := 0)`.

Variadic calls such as `WriteLn` and `Format` are entirely positional and reject
names (FP3026), including a named prefix followed by a positional tail (FP2016).
`Some`, `Ok`, and `Error` accept their single payload by position or with the
canonical field label `Value`, for example `Some(Value := 3)`.

```pascal
function Double(N: integer): integer;
begin
  return N * 2;
end function;

const F: function(Value: integer): integer := Double;
const Answer: integer := F(Value := 3);
const Same: integer := F(3);
```

Here `Double(N := 3)` and `F(Value := 3)` are valid; `F(N := 3)` is rejected
(FP3025). Assignment and callback compatibility compare parameter positions,
types and modes, result types, and variadic status; names do not affect
compatibility and never reorder arguments during assignment. Inferred callable
bindings retain their inferred signature's labels, including labels inside
collections and decision expressions. When compatible branches differ only in
labels, the inferred signature retains the first branch's labels. An explicit
receiving signature determines the labels of calls through that receiving value.

## See also

- [Declarations](declarations.md)
