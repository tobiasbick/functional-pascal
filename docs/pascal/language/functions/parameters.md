# Parameters

Parameters are separated by semicolons in declarations. Calls use commas. Each parameter requires a type annotation:

```pascal
program ClampExample;

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
  var R: integer := Clamp(150, 0, 100);  // 100
end program;
```

## See also

- [Declarations](declarations.md)
- [Mutable parameters](mutable-parameters.md)
