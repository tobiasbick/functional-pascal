# Nested functions

Functions and procedures can be declared inside other routines. Use nested
declarations for local helpers and access to enclosing parameters or locals:

```pascal
function Hypotenuse(A: real; B: real): real;

  function Square(X: real): real;
  begin
    return X * X;
  end function;

begin
  return Sqrt(Square(A) + Square(B));
end function;
```

## Mutual recursion

All named sibling signatures are available throughout their lexical scope.
Mutually recursive routines may stand beside each other:

```pascal
function IsEven(N: integer): boolean;
begin
  if N = 0 then
    return true;
  else
    return IsOdd(N - 1);
  end if;
end function;

function IsOdd(X: integer): boolean;
begin
  if X = 0 then
    return false;
  else
    return IsEven(X - 1);
  end if;
end function;
```

Nested routines that escape as first-class values capture their enclosing environment.
See [Capturing closures](closures.md).

An inner named routine shadows an outer name throughout the inner scope.
Named helpers may refer to later enclosing locals, but call or capture the
helper only after its dependencies have initialized. The compiler also checks
indirect dependencies through other helpers. See
[declaration order](../types/declaration-order.md).

## See also

- [Declarations](declarations.md)
- [Capturing closures](closures.md)
