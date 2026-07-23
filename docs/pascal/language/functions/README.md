# Functions

Functions and procedures are the primary building block in Functional Pascal. They can be stored in variables, passed as arguments, and nested inside other functions.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_decl`, `procedure_decl`, `function_type`, `procedure_type`).

| Topic | Description |
|-------|-------------|
| [Declarations](declarations.md) | `function` / `procedure` shape and early exit |
| [Parameters](parameters.md) | Parameter lists and call syntax |
| [Mutable parameters](mutable-parameters.md) | `mutable` on parameters |
| [Function types](function-types.md) | Callable type expressions |
| [First-class functions](first-class.md) | Variables and higher-order calls |
| [Nested functions](nested.md) | Local helpers and mutual recursion |
| [Capturing closures](closures.md) | Anonymous callables with lexical environments |
| [Generic routines](generic-routines.md) | Type parameters on routines |
| [Early return](early-return.md) | `return` exits immediately |
| [Postfix chaining](postfix-chaining.md) | `.Field`, `[Index]`, and `.Method(args)` on expression results |

## Runtime recursion limit

The VM bounds both intermediate value storage and active function call frames. Excessive
recursion therefore stops with a `Call stack overflow` runtime diagnostic instead of exhausting
host memory. Reduce the recursion depth or rewrite the processing as a loop when this occurs.

## See also

- [Types — generics](../types/generics.md)
- [Basics](../basics/README.md)
- [Record methods](../types/record-methods.md)
