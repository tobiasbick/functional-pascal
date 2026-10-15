# Generic routines

Functions and procedures can declare type parameters in angle brackets after the name:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_params` on `function_heading` and `procedure_heading`).

```pascal id=example-1 fragment context=declarations-console
function Identity<T>(Value: T): T;
begin
  return Value;
end function;

procedure PrintValue<T>(Value: T);
begin
  Console.WriteLn(Value);
end procedure;
```

Type arguments are inferred from supplied values first:

```pascal id=example-2 fragment context=generic-calls
begin
  Console.WriteLn(Identity(42)); // T = integer
  Console.WriteLn(Identity('hello')); // T = string
  PrintValue(3.14); // T = real
end.
```

An expected result type fills only parameters still unresolved after the
arguments have contributed. Context comes from an annotation, assignment,
return, routine argument, constructor field, or typed callable signature.
Direct calls, record instance/static methods, and callable values use the same
rule, including imported routines. Named argument order does not change inference.

```pascal id=example-3
program ContextualFactory;

function Empty<T>(): option of T;
begin
  return None;
end function;

function Missing(): option of integer;
begin
  return Empty();
end function;

begin
  const Value: option of integer := Empty();
  const Factory: function(): option of integer := Empty;
  const Copy := Factory();
  if not (Value is None) or not (Copy is None) or not (Missing() is None) then
    panic('expected empty values');
  end if;
end.
```

An already determined argument type cannot be replaced by the expected result.
`const Value: real := Identity(1);` is rejected because the argument fixes T to
integer. Use a real literal or an explicit `Std.Conv.IntToReal` conversion.
Constraints are checked after inference, including parameters supplied by context.
Every declared type parameter needs evidence: a result-only factory without context
and a parameter unused by both arguments and result are rejected before lowering.
Enclosing generic parameters retain their own identities, even when a nested
routine declares another parameter with the same source name.

Multiple type parameters are separated by commas:

```pascal id=example-4 fragment context=declarations
function First<A, B>(X: A; Y: B): A;
begin
  return X;
end function;
```

See [Generics](../types/generics.md) for constraints and method-level generics on record methods.

For spawned calls, the receiving task annotation does not specialize the
callee. Resolve its arguments or use an explicitly typed callable before
creating the handle; see [Task handles](../concurrency/task-handles.md).

## See also

- [Generics](../types/generics.md)
