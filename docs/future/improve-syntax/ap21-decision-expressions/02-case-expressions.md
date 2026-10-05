# AP21.2: case expressions

Package: [AP21: Decision expressions](README.md)

## Scope

Add `case Value of when Label: Expression; ... end case` as an expression,
with AP03 coverage rules.

## Prerequisites

- AP21.1 (shared branch-type rules).
- AP13.5 (`when` arms).
- AP03.2 (closed-enum coverage without `else`).

## Implementation

- Parser: `case` in expression position; each arm is one expression ended by
  `;`; optional `else` arm for open domains.
- Sema: coverage as for `case` statements; pattern bindings visible in the arm
  expression; branch-type compatibility from AP21.1.
- Compiler: reuse case lowering with a value result.
- Diagnostics: missing variants, missing `end case`, statements in an arm.

## Affected areas

- Parser expressions, sema decision expression module and exhaustiveness,
  `crates/fpas-compiler/src/lowering/case/`, `fpas-fmt`.

## Migration

None.

## Documentation

- `docs/pascal/language/control-flow/case-of-intro.md`, pattern-matching
  pages, `docs/specs/grammar.ebnf`.

## Verification

- `case` expressions over enums, `Option`, `Result`, integers, and strings;
  bindings; guards; missing variants; incompatible results; `return case`.
