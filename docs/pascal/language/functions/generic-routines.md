# Generic routines

Functions and procedures can declare type parameters in angle brackets after the name:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_params` on `function_heading` and `procedure_heading`).

```pascal
uses Std.Console as Console;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

procedure PrintValue<T>(Value: T);
begin
  Console.WriteLn(Value);
end procedure;
```

Type arguments are inferred from the call-site arguments:

```pascal
program Example;

uses Std.Console as Console;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

procedure PrintValue<T>(Value: T);
begin
  Console.WriteLn(Value);
end procedure;

begin
  Console.WriteLn(Identity(42)); // T = integer
  Console.WriteLn(Identity('hello')); // T = string
  PrintValue(3.14); // T = real
end program;
```

Multiple type parameters are separated by commas:

```pascal
function First<A, B>(X: A; Y: B): A;
begin
  return X;
end function;

```

See [Generics](../types/generics.md) for constraints and method-level generics on record methods.

## See also

- [Generics](../types/generics.md)
