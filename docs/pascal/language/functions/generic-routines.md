# Generic routines

Functions and procedures can declare type parameters in angle brackets after the name:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`generic_params` on routines).

```pascal
function Identity<T>(Value: T): T;
begin
  return Value
end;

procedure PrintValue<T>(Value: T);
begin
  WriteLn(Value)
end;
```

Type arguments are inferred from the call-site arguments:

```pascal
begin
  WriteLn(Identity(42));       // T = integer
  WriteLn(Identity('hello'));  // T = string
  PrintValue(3.14)             // T = real
end.
```

Multiple type parameters are separated by commas:

```pascal
function First<A, B>(X: A; Y: B): A;
begin
  return X
end;
```

See [Generics](../types/generics.md) for constraints and method-level generics on record methods.

## See also

- [Generics](../types/generics.md)
