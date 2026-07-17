# Nested functions

Functions can be declared inside other functions. Use nested declarations for local helpers and mutual recursion:

```pascal
function Hypotenuse(A: real; B: real): real;

  function Square(X: real): real;
  begin
    return X * X;
  end;

begin
  return Sqrt(Square(A) + Square(B));
end;
```

## Mutual recursion

Declare callees before callers when only one direction of call is needed. For mutual recursion, nest the helper in the outer routine so both names are in scope when bodies are checked:

```pascal
function IsEven(N: integer): boolean;
  function IsOdd(X: integer): boolean;
  begin
    if X = 0 then
      return false
    else
      return IsEven(X - 1)
  end;
begin
  if N = 0 then
    return true
  else
    return IsOdd(N - 1)
end;
```

Nested routines that escape as first-class values capture their enclosing environment.
See [Capturing closures](closures.md).

## See also

- [Declarations](declarations.md)
- [Capturing closures](closures.md)
