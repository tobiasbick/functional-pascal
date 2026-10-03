# Function types

Function types describe the signature of a callable:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_type`, `procedure_type`).

```pascal
type IntBinaryOp = function(A: integer; B: integer): integer;

type StringAction = procedure(S: string);

```

Compatibility is positional. Parameter names describe the signature but do not
need to match the assigned routine's parameter names. Parameter types, mutable
parameter modes and the function result type must match. Calling a callable value
uses the same arity and argument-type checks as calling a named routine.

A procedure type describes a value that can be stored and invoked. A procedure
call itself produces no value. Function results require a consumer or explicit
`discard`; see [first-class callables](first-class.md#result-consumption-and-discard).

## See also

- [First-class functions](first-class.md)
- [Type aliases](../types/type-aliases.md)
