# Remove: `forward` Declarations

**Status: done** (implemented).

## Removed

- The `forward` keyword (`forward` is a normal identifier).
- Forward-declaration syntax: `function Foo(...): T; forward;`
- Pending-forward tracking and matching in semantic analysis.

## Rationale

Mutual recursion is expressed by nesting helpers or by ordering declarations so callees appear first when only one call direction is required.

## Implementation notes

- **Lexer:** `forward` removed from the keyword set (62 keywords).
- **Parser:** `function` / `procedure` bodies always use `begin` … `end`. Interface signatures use an internal `FuncBody::SignatureOnly` in the AST (not source syntax).
- **Sema:** routine registration is a single define per name in a scope; duplicates are errors.
- **Grammar (EBNF):** `docs/specs/grammar.ebnf` — keyword list and `function_decl` / `procedure_decl`.
- **Docs:** `docs/pascal/01-overview.md`, `docs/pascal/04-functions.md`.
