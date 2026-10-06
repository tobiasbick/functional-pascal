# AP13.6: Expression closers

Package: [AP13: Explicit block boundaries](README.md)

## Scope

Close anonymous routines with `end function` / `end procedure` and record
updates with `end with`. Expressions take no terminating `;` of their own.

## Prerequisites

- AP13.3 (named routine closers).

## Implementation

- Parser: anonymous routine expressions end with their named closer; an
  anonymous routine argument ends before the call's `)` without `;`; record
  updates are `P with X := 1; end with`.
- Reject an extra `;` between an anonymous routine and the closing parenthesis.
- Formatter: expression closers inside arguments and declarations.

## Affected areas

- `crates/fpas-parser/src/parser/expr/closure.rs`, `expr/records.rs`,
  `parser/block_closers.rs`, and `stmt/terminators.rs` (ownership and recovery).
- `crates/fpas-fmt/src/emit/expr/` and `src/comments/traversal/`.

## Migration

Rewrite every anonymous routine and record update in all repository consumers.

## Documentation

- `docs/specs/grammar.ebnf` (`closure_expr`, `record_update`),
  `docs/pascal/language/functions/closures.md`,
  `docs/pascal/language/types/record-update.md`, `fmt-style.md`.

## Verification

- Expression endings in declarations, returns, and call arguments; nested
  anonymous routines; record updates inside arguments; the rejected extra `;`.

## Result

Implemented on `codex/syntax-changes-2`. Anonymous routines and record updates
require their matching named ending, without an expression-owned terminator.
Diagnostics distinguish expression endings from terminated declarations and
statements. Recovery preserves enclosing endings and argument boundaries.

Formatting retains ending and field comments, indents closure bodies inside
record fields, and remains idempotent. Repository sources, embedded fixtures,
current documentation, authoring guidance, and editor snippets and indentation
are migrated. Regenerated intrinsic editor declarations have no content changes.
Parser, formatter, CLI, snippet, and execution regressions preserve captures,
record-update evaluation order, nested scopes, and existing literal behavior.

Build, Rust/FPAS formatting, editor checks, documentation links, all 22 example
and app projects, and 461 FPAS tests pass; one FPAS test is intentionally skipped.
Workspace verification and targeted reruns resolve every delivery-related failure.
The known VM socket-timeout-resolution failure reproduces on the unchanged
AP13.2 baseline. The unmodified LSP protocol test can hang while awaiting an
unanswered capability-registration request; the hang also reproduces on the
unchanged AP13.5 baseline, while an acknowledged registration and shutdown pass.

All AP13 work packages are complete. Record literals retain their existing form;
their replacement belongs to AP10. Decision expressions and task scopes remain
in AP21 and AP26, and their future-only shared checks are outside this delivery.
