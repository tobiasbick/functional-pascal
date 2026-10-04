# Pattern matching

Statement and value-producing `case of` forms match values against static
constants, ranges, and nested variant patterns. `const Name` binds a matched
value; `_` ignores a payload. Variant names include their declaring type.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_arm`, `case_label`).

| Topic | Description |
|-------|-------------|
| [Syntax](syntax.md) | Arm shape and separators |
| [Scalar labels](scalar-labels.md) | Values, commas, `else`, block arms |
| [Ranges](ranges.md) | `a..b` labels |
| [Enum patterns](enum-patterns.md) | Plain and data-carrying enums |
| [Result and Option patterns](result-option-patterns.md) | `Result.Ok` / `Result.Error` / `Option.Some` / `Option.None` |
| [Guards](guards.md) | Conditions after matching and explicit arm-local bindings |
| [Exhaustiveness](exhaustiveness.md) | Compile-time coverage rules |

## See also

- [Control flow — case intro](../control-flow/case-of-intro.md)
- [Error handling](../error-handling/README.md)
