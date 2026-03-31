# Remove: `forward` Declarations

> Priority: 1 — no dependencies, isolated change.

## What to remove

- The `forward` keyword.
- Forward-declaration syntax: `function Foo(...): T; forward;`
- The forward-reference resolution pass in the compiler.

## Current state

No example uses `forward`. Mutual recursion is not needed in computation or
event-loop code. Functions can be reordered to avoid forward references.

## Scope

- **Lexer/token:** remove `forward` keyword.
- **Parser:** remove the `'forward' ';'` branch in `function_decl` /
  `procedure_decl`.
- **Sema:** remove forward-reference tracking and resolution.
- **Grammar (EBNF):** remove `forward` from keyword list and function
  declaration alternatives.
- **Docs:** remove from `01-overview.md` keyword list and `04-functions.md`.
