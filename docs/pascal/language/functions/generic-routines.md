# Generic routines

Functions and procedures can declare type parameters in angle brackets after the name:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_params` on `function_heading` and `procedure_heading`).

```pascal
uses Std.Console as Console;

function Identity of (T)(Value: T): T;
begin
  return Value;
end function;

procedure PrintValue of (T)(Value: T);
begin
  Console.WriteLn(Value);
end procedure;
```

Type arguments are inferred from the call-site arguments:

Empty collections and the absent type component of an Option or Result
constructor receive context from the other arguments, including inside nested
collections. Every concrete component participates in constraint checking;
argument order cannot hide a resource or callable component from `Equatable`.

```pascal
program Example;

uses Std.Console as Console;

function Identity of (T)(Value: T): T;
begin
  return Value;
end function;

procedure PrintValue of (T)(Value: T);
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
function First of (A, B)(X: A; Y: B): A;
begin
  return X;
end function;

```

See [Generics](../types/generics.md) for constraints and method-level generics on record methods.

## Forwarding generic arguments

A generic routine can pass its parameters to another generic routine. Type
inference preserves the caller's type parameter, including inside collections
and callable signatures; the two routines need not use the same parameter name.

```pascal
function Twice of (T: Numeric)(Value: T): T;
begin
  return Value + Value;
end function;

function Forward of (U: Numeric)(Value: U): U;
begin
  return Twice(Value);
end function;
```

The caller's declared constraint must guarantee the callee's constraint. An
unconstrained `U`, or `U: Comparable`, cannot be passed to `Twice`: the call is a
compile-time constraint error even when the caller is never instantiated.
`Numeric` also guarantees `Comparable`, `Equatable` and `Printable`;
`Comparable` guarantees `Equatable` and `Printable`, and `Equatable` guarantees
`Printable`. Independent caller type parameters remain distinct even when they
have the same constraint.

## See also

- [Generics](../types/generics.md)
