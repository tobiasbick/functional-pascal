# AP21.1: if expressions

Package: [AP21: Decision expressions](README.md)

## Scope

Add `if C then A elsif D then B else E end if` as an expression.

## Prerequisites

- AP13.4 (`if` statement syntax with `elsif` and `end if`).
- AP07.3 (condition precedence).
- The branch-type and expression/statement distinction decisions in the
  [package README](README.md#open-decisions).

## Implementation

- Parser: `if` in expression position; specify and implement how an
  expression `if` is distinguished from the statement form at the start of a
  statement.
- Sema: `else` required; branch-type compatibility rules (specify them in the
  current documentation); single expressions only.
- Compiler: only the selected branch is evaluated.
- Diagnostics: missing `else`, missing `end if`, statements inside a branch.
- Formatter: single-line and multi-line layout.

## Affected areas

- Parser expressions, `crates/fpas-sema/src/check/expr/` (a focused decision
  expression module), compiler expression lowering, `fpas-fmt`.

## Migration

None; optional adoption in examples.

## Documentation

- `docs/pascal/language/control-flow/if-then-else.md`, `operators.md`
  (precedence position), `docs/specs/grammar.ebnf`.

## Verification

- Nested `if` expressions, `elsif` chains, incompatible branch types, missing
  `else`, statements in branches, evaluation of only the selected branch.
