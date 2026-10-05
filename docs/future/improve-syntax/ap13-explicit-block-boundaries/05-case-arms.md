# AP13.5: Case arms

Package: [AP13: Explicit block boundaries](README.md)

## Scope

Introduce `case V of when Labels [if Guard]: ... else ... end case;` with
statement-list arms.

## Prerequisites

- AP13.1 (`when` reserved).
- AP13.4 (statement-list branches and `null;`).

## Implementation

- Parser: each arm starts with `when`; the next `when`, `else`, or `end case`
  ends it. The catch-all is `else`.
- Sema: each arm is a scope; existing pattern and exhaustiveness checks are
  unchanged.
- Formatter: arm indentation.

## Affected areas

- `crates/fpas-parser/src/parser/stmt/branching.rs` (case parsing).
- `crates/fpas-sema/src/check/stmt/control_flow/if_case/`.
- `crates/fpas-fmt/src/emit/stmt/`; editor snippets.

## Migration

Rewrite every `case` statement in all repository consumers.

## Documentation

- `docs/specs/grammar.ebnf` (`case_stmt`, `case_arm`),
  `docs/pascal/language/control-flow/case-of-intro.md`,
  `docs/pascal/language/pattern-matching/`.

## Verification

- Arms with multiple labels, guards, patterns, `null;` arms, `else`, nested
  `case` inside arms, missing `end case`, comments between arms.
