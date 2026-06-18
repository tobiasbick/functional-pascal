# Result

`Result of T, E` represents either a success (`Ok`) or a failure (`Error`):

```pascal
var R: Result of integer, string := Ok(42);
var E: Result of integer, string := Error('not found');
```

## Returning errors

```pascal
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
```

## Handling with case

Use `case of` with destructuring to handle both branches:

```pascal
var R: Result of integer, string := Divide(10, 0);
case R of
  Ok(V):  WriteLn('Value: ' + IntToStr(V));
  Error(E): WriteLn('Error: ' + E);
end;
```

The binding variable (`V`, `E`) is scoped to its arm body.

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Std.Result`](../../std/result.md)
