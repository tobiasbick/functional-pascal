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
    return Min
  else if Value > Max then
    return Max
  else
    return Value;
end;

begin
  var R: integer := Clamp(150, 0, 100);  // 100
end.
```

## See also

- [Declarations](declarations.md)
- [Mutable parameters](mutable-parameters.md)
