# Generic routines

Functions and procedures declare type parameters with `of (...)` after the name:

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

Every declared type parameter must receive one complete argument type. A result
annotation does not supply routine type arguments, and unused type parameters
cannot be inferred.

A `pure function` additionally requires every concrete type argument to contain
only resource-free data or pure callables. An ordinary generic routine can require
a pure callback in its explicit signature; type parameters used by that callback
inherit its parameter/result requirements. This requirement also applies through
generic record fields and aliases. Contextual instantiation of a generic callable
preserves its declared purity and checks these requirements before it can be stored.

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

See [Generics](../types/generics.md) for constraints and generic records and enums.

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

Nested generic declarations may reuse a type parameter name. The inner parameter
belongs to the inner declaration and cannot accept an outer parameter's value
merely because both are spelled `T`. Captured values and previously resolved
signatures keep their enclosing types. Inferring an inner call replaces only
its declared parameters, including in collections and returned callable types.

A generic routine value can receive a concrete callable signature from an
explicit binding annotation or a callback parameter. Its parameters are
instantiated from that signature and must satisfy their declared constraints.
An unresolved generic callback does not determine a caller's type arguments;
the other arguments must supply enough context, or the callback needs an explicit
callable annotation. Callable parameter names do not affect this matching.

The same requirement applies inside arrays, dictionaries, records and variants.
For example, `const Callbacks := [Identity];` needs an annotation such as
`array of (function(Value: integer): integer)`. Later invocations cannot make the
stored generic routine polymorphic.

## See also

- [Generics](../types/generics.md)
