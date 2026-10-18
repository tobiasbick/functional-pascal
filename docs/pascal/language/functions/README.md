# Functions

Functions and procedures are the primary building block in Functional Pascal. They can be stored in variables, passed as arguments, and nested inside other functions.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_decl`, `procedure_decl`, `function_type`, `procedure_type`).

| Topic | Description |
|-------|-------------|
| [Declarations](declarations.md) | `function` / `procedure` shape and early exit |
| [Parameters](parameters.md) | Parameter lists, call syntax, and named arguments |
| [Preconditions and postconditions](contracts.md) | Boolean `requires` entry checks and `ensures` return checks |
| [`var` parameters](var-parameters.md) | Parameters that change the caller's variable |
| [Function types](function-types.md) | Callable type expressions |
| [First-class functions](first-class.md) | Variables and higher-order calls |
| [Nested functions](nested.md) | Local helpers and mutual recursion |
| [Capturing closures](closures.md) | Anonymous callables with lexical environments |
| [Generic routines](generic-routines.md) | Type parameters on routines |
| [Discarding values](discard.md) | Explicit value disposal and task ownership |
| [Early return](early-return.md) | `return` exits immediately |
| [Postfix chaining](postfix-chaining.md) | `.Field`, `[Index]`, and `.Method(args)` on expression results |
| [Receiver calls](fluent-calls.md) | Call declared record members or fixed native type operations |

## Runtime recursion limit

The VM bounds both intermediate value storage and active function call frames. Excessive
recursion therefore stops with a `Call stack overflow` runtime diagnostic instead of exhausting
host memory. Reduce the recursion depth or rewrite the processing as a loop when this occurs.

An eligible direct tail call (`return F(X)`, or a procedure ending with a call
to another procedure) reuses the current frame and does not add a frame toward
this limit. Pending exit work can prevent that conversion: an `ensures` clause
must check the returned value, and an active task scope must complete its exit.
The return syntax alone therefore does not guarantee frame reuse. The debugger
keeps tail-call frames visible and still applies the limit.

## See also

- [Types — generics](../types/generics.md)
- [Basics](../basics/README.md)
- [Record methods](../types/record-methods.md)
