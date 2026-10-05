# AP07.3: Logical precedence and mixing rules

Package: [AP07: Boolean rules](README.md)

## Scope

Implement the Q03 precedence table: comparisons above `not`, `not` above the
binary logical operators, same-operator logical chains only, and
non-associative comparisons.

## Prerequisites

None. AP07.1 is recommended first so that migrated expressions are tested with
the final evaluation rule.

## Implementation

- Parser: restructure the precedence levels; reject mixed logical chains with
  a diagnostic showing the parenthesized form; keep comparison-chain rejection
  with the `A < B and B < C` hint.
- Formatter: mirror precedence and required parentheses.
- Debugger expression parsing follows the same table.

## Affected areas

- `crates/fpas-parser/src/parser/expr/precedence.rs`.
- `crates/fpas-fmt/src/emit/expr/`.
- `crates/fpas-vm/src/vm/debug/` expression translation.

## Migration

- Use the old parser's AST to find every logical expression whose grouping
  changes; add explicit parentheses that preserve the old meaning where it was
  intended, and record any expression that was a latent bug.
- Cover Rust-embedded fixtures and documentation examples.

## Documentation

- `docs/specs/grammar.ebnf` (expression productions) and
  `docs/pascal/language/basics/operators.md` with the complete table.

## Verification

- Parser tests for every precedence boundary, all mixed logical pairs,
  parentheses, `not` placement, and comparison chains.
- Formatter round trip; FPAS suite unchanged.
