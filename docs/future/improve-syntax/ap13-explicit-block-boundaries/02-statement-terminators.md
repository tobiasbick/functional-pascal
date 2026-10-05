# AP13.2: Statement and declaration terminators

Package: [AP13: Explicit block boundaries](README.md)

## Scope

Change `;` from a statement separator to a terminator: every statement and
declaration ends with `;`, including the last one before `end`, `else`, or
`until`. The program's final `end.` is the only exception. Block closers stay
as they are in this work package.

## Prerequisites

- AP02 (diagnostic codes).

## Implementation

- Parser: rework `statement_list` so each statement requires its `;`. Reject a
  missing final `;` with a diagnostic and recover at the next statement.
- Apply the agreed rule before `else` as well: the statement that ends a
  branch is terminated by `;` before `else`, which the old separator grammar
  rejected. Diagnose a missing `;` there.
- Formatter: emit every terminator.

## Affected areas

- `crates/fpas-parser/src/parser/` (statement lists, `stmt/`).
- `crates/fpas-fmt/src/emit/stmt/`, comment traversal.

## Migration

Add missing terminators in every repository consumer. The formatter on the
branch can perform the rewrite once the parser accepts both forms there; the
merged parser accepts only the new form.

## Documentation

- `docs/specs/grammar.ebnf` (`statement_list`), `fmt-style.md`, control-flow
  and declaration pages, authoring skill.

## Verification

- Missing final `;` in routine bodies, loops, branches, and `repeat`;
  terminators before `until`; comments before closers.
- Corpus round trip and idempotence; unchanged FPAS suite.
