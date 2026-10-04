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
  const R: integer := Clamp(150, 0, 100);  // 100
end program;
```

Ordinary parameters are read-only snapshots. To reassign a local copy, declare
`var Local: Type := Parameter;` inside the body. Explicit
[var parameters](var-parameters.md) instead update the caller's selected storage
and require a matching `var` marker at the call site.

## See also

- [Declarations](declarations.md)
- [Var parameters](var-parameters.md)
