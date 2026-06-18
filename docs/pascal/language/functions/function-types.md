# Function types

Function types describe the signature of a callable:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_type`, `procedure_type`).

```pascal
type
  IntBinaryOp = function(A: integer; B: integer): integer;
  StringAction = procedure(S: string);
```

## See also

- [First-class functions](first-class.md)
- [Type aliases](../types/type-aliases.md)
