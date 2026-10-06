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

- `crates/fpas-parser/src/parser/stmt/case/` (case parsing and labels),
  `parser/block_closers.rs`, and `stmt/terminators.rs` (boundaries and recovery).
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

## Result

Implemented locally on `codex/syntax-changes-2`. Every case arm starts with
`when`, contains a nonempty scoped statement list, and shares the statement's
`end case;` with the optional final `else` arm. An otherwise empty arm body
requires `null;`.
Explicit compound statements retain an additional scope. Existing label,
pattern, guard, and exhaustiveness behavior is preserved.

Catch-all locals are scoped consistently in semantic analysis, scalar and
variant lowering, and closure capture discovery. Parser recovery preserves
the next `when` arm or an enclosing named ending. Formatting preserves
comments, explicit blocks, nested cases, and every required terminator.

Repository sources, embedded fixtures, current documentation, authoring
guidance, editor snippets and indentation, and lexer expectations are migrated.
Parser, scope, formatter, CLI, and execution regressions cover the new syntax,
guard evaluation order, arm visibility, returns, loop control, and captures.

Build, Rust/FPAS formatting, editor grammar/contracts/compilation, documentation
links, the generated compiler benchmark source, all 22 example/app projects,
and 460 FPAS tests pass; one FPAS test remains intentionally skipped. Across
the workspace suite and corrected lexer fixture rerun, 3,434 Rust tests pass.
The remaining VM socket-timeout-resolution failure also reproduces on the
unchanged AP13.2 baseline and is independent of this delivery.

Expression closers are delivered by AP13.6. Closed-enum catch-all restrictions and
explicit pattern-binding syntax remain in AP03 and AP20 respectively.
