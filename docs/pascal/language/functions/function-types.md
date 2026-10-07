# Function types

Function types describe the signature of a callable:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_type`, `procedure_type`).

```pascal
type IntBinaryOp = function(A: integer; B: integer): integer;

type StringAction = procedure(S: string);

type Step = procedure(var Value: integer);
```

A parameter declared with `var` changes the caller's variable; see
[`var` parameters](var-parameters.md). Parameter modes are part of the type:
`procedure(Value: integer)` and `procedure(var Value: integer)` are different
types, and neither routine can be assigned to the other type.

## See also

- [First-class functions](first-class.md)
- [Type aliases](../types/type-aliases.md)
