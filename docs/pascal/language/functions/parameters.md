# Parameters

Parameters are separated by semicolons in declarations. Calls use commas. Each parameter requires a type annotation:

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
