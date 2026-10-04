# Result

`Result of (T, E)` represents either a success (`Result.Ok`) or a failure (`Result.Error`):

```pascal
var R: result of (integer, string) := Result.Ok(42);
var E: result of (integer, string) := Result.Error('not found');

```

## Returning errors

```pascal
function Divide(A: integer; B: integer): result of (integer, string);
begin
  if B = 0 then
    return Result.Error('Division by zero');
  else
    return Result.Ok(A div B);
  end if;
end function;

```

## Handling with case

Use `case of` with destructuring to handle both branches:

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

var R: result of (integer, string) := Divide(10, 0);
case R of
  when Result.Ok(const V):
    Console.WriteLn('Value: ' + Conv.IntToStr(V));
  when Result.Error(const E):
    Console.WriteLn('Error: ' + E);
end case;
```

The binding variable (`V`, `E`) is scoped to its arm body.

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Std.Results`](../../std/result/result.md)
