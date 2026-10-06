# Result

`Result of T, E` represents either a success (`Ok`) or a failure (`Error`):

```pascal
var R: Result of integer, string := Ok(42);
var E: Result of integer, string := Error('not found');
```

## Returning errors

```pascal
function Divide(A: integer; B: integer): result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero');
  else
    return Ok(A div B);
  end if;
end function;
```

## Handling with case

Use `case of` with destructuring to handle both branches:

```pascal
var R: result of integer, string := Divide(10, 0);
case R of
  when Ok(V):
    WriteLn('Value: ' + IntToStr(V));
  when Error(E):
    WriteLn('Error: ' + E);
end case;
```

The binding variable (`V`, `E`) is scoped to its arm body.

## Consuming results

A function returning `Result` cannot be called as a standalone statement.
Handle the branches with `case`, or use [try](try.md) while consuming the
success value. If ignoring success and failure is intentional, write
`discard Divide(10, 0);`. This evaluates the call without unwrapping it.
Results containing task handles or unverified callable captures cannot be
discarded; see [discarding values](../functions/discard.md).

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Pattern matching — Result and Option](../pattern-matching/result-option-patterns.md)
- [Try operator](try.md)
- [`Std.Results`](../../std/result/result.md)
