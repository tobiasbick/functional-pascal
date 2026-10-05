# Function types

Function types describe the signature of a callable:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_type`, `procedure_type`).

```pascal
type IntBinaryOp = function(A: integer; B: integer): integer;

type StringAction = procedure(S: string);

```

Compatibility is positional. Parameter names describe the signature but do not
need to match the assigned routine's parameter names. Parameter types, value/var
parameter modes and compatible parameter/result types must match. Calling a callable value
uses the same arity and argument-type checks as calling a named routine.

For example, `procedure(var Value: integer)` requires `Action(var Count)`;
it is incompatible with `procedure(Value: integer)`. Var mode is retained through
returned functions, stored fields, indexing and compiled unit interfaces.

A procedure type describes a value that can be stored and invoked. A procedure
call itself produces no value. Function results require a consumer or explicit
`discard`; see [first-class callables](first-class.md#result-consumption-and-discard).

## Explicit purity

Write `pure function` on a named declaration, anonymous function, or function
type to require a purity guarantee:

```pascal
pure function ApplyTwice(F: pure function(Value: integer): integer; Value: integer): integer;
begin
  return F(F(Value));
end function;
```

Pure functions call only pure functions. Their parameters and results contain
only resource-free data and pure callables, recursively through records, enums,
Options, Results and collections. Ordinary functions, procedures, resource
handles, channels and tasks do not satisfy this requirement. Pure functions have
no `var` parameters. They can update their own local data but cannot read mutable
outer bindings, capture mutable cells or start tasks. Immutable outer data and
pure callables can be read and captured. A panic or nontermination does not
violate purity.

A pure function can be assigned to an ordinary function type. The destination
then exposes only an ordinary callable; its original guarantee cannot be recovered
by assigning it back to a pure type. Purity is explicit, never inferred from a
body or a `const` binding. Procedures have no pure form.

Callable parameters are contravariant and results are covariant: a function that
accepts any ordinary callback can satisfy a type whose caller supplies only pure
callbacks. The reverse is invalid. `var` parameter types and channel element
types remain invariant, because those paths share writable storage. Nominal
record and enum type arguments also remain invariant: changing an argument can
reverse the requirement of a callable field that consumes it.

## See also

- [First-class functions](first-class.md)
- [Type aliases](../types/type-aliases.md)
