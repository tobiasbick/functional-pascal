# Remove: Nested Patterns and Wildcards

> Priority: 4 — affects pattern matching subsystem only.

## What to remove

- Multi-level nested patterns: `Expr.Add(Expr.Num(A), Expr.Num(B))`
- Wildcard `_` in pattern positions.
- Literal matching inside destructuring patterns.

## What stays

- **Simple one-level enum destructuring:** `Shape.Circle(R)`,
  `Shape.Rectangle(W, H)`, `Shape.Point`
- **Result/Option destructuring:** `Ok(V)`, `Error(E)`, `Some(X)`, `None`
- **Guard clauses:** `N if N > 0:`
- **Scalar matching:** values, ranges, comma-separated labels

One-level destructuring covers all practical patterns in event-loop and
computation code.

## Scope

- **Parser:** simplify `pattern_arg` — only allow `identifier` (binding),
  remove recursive `enum_pattern`, `literal_pattern`, and `_`.
- **Sema:** remove nested pattern type-checking.
- **Compiler:** remove nested pattern code generation.
- **Grammar (EBNF):** simplify `pattern_arg` production.
- **Docs:** simplify `06-pattern-matching.md`. Remove nested pattern section.
  Remove `examples/pascal/pattern-matching/nested.fpas`.
