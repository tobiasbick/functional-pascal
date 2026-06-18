# Pattern matching

The `case of` statement matches a value against constants, ranges, enum variants, and destructuring patterns.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_arm`, `case_label`).

| Topic | Description |
|-------|-------------|
| [Syntax](syntax.md) | Arm shape and separators |
| [Scalar labels](scalar-labels.md) | Values, commas, `else`, block arms |
| [Ranges](ranges.md) | `a..b` labels |
| [Enum patterns](enum-patterns.md) | Plain and data-carrying enums |
| [Result and Option patterns](result-option-patterns.md) | `Ok` / `Error` / `Some` / `None` |
| [Guards](guards.md) | `label if cond:` and scalar bindings |
| [Exhaustiveness](exhaustiveness.md) | Compile-time coverage rules |

## See also

- [Control flow — case intro](../control-flow/case-of-intro.md)
- [Error handling](../error-handling/README.md)
