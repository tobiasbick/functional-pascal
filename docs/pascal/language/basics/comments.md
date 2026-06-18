# Comments

Three comment styles are supported. Comments do **not** nest. Shared declarations belong in units imported via `uses`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (Part 1 — comments).

```pascal
{ Brace comment — single or multi-line }

(* Parenthesis-star comment — single or multi-line *)

// Line comment — to end of line

/// Doc line comment — same as `//`, preserved by `fpas fmt` when attached to declarations
```

`{ outer { inner } ← closes here` — the first `}` ends the comment.

## See also

- [Formatter style](../../tools/fmt-style.md)
- [Units](../program-structure/units.md)
