# Function types

Function types describe the signature of a callable:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_type`, `procedure_type`).

```pascal id=example-1 fragment context=declarations
type IntBinaryOp = function(A: integer; B: integer): integer;

type StringAction = procedure(S: string);

type Step = procedure(var Value: integer);
```

A parameter declared with `var` changes the caller's variable; see
[`var` parameters](var-parameters.md). Parameter modes are part of the type:
`procedure(Value: integer)` and `procedure(var Value: integer)` are different
types, and neither routine can be assigned to the other type.

Parameter names are required and unique without regard to case, including
names in nested callable types. They are the labels for named calls through
that static type: a value of `function(A: integer; B: integer): integer`
accepts `F(B := 2, A := 10)` or `F(10, 2)`. Each call uses one form throughout
and supplies every parameter once.

Names do not participate in assignment or callback compatibility. A routine
with parameters `Left` and `Right` may implement this type; calls through the
receiving value still use `A` and `B`. No runtime lookup of implementation
parameter names occurs. See [named arguments](parameters.md#named-arguments).

## See also

- [First-class functions](first-class.md)
- [Type aliases](../types/type-aliases.md)
