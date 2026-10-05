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

### Confirmed transition rule

Until named control-flow closers are introduced by AP13.4, `if`, `for`, and
`while` keep their single-statement bodies. The final body's `;` also
terminates the enclosing control statement; no second `;` is added. Each
branch is terminated before `else`. Existing nearest-unmatched-`if` ownership
is preserved; explicit compound statements disambiguate nested branches.

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

## Result

Implemented locally on `codex/syntax-changes-2` after the local AP02 and
AP13.1 changes. Statements require terminators at every list boundary,
including before `else` and `until`; declarations retain their required
terminators. Single-statement control bodies use the confirmed transition
rule above. The program keeps `end.`, and expression closers have no
terminator of their own.

Repository sources, embedded fixtures, source generators, formatter goldens,
CLI templates, editor snippets, and documentation examples are migrated.
Conversion used temporary parser-guided tools under the ignored
`.temp-data/` directory; the parser has no legacy acceptance mode.
Named closers remain in the later work packages. The package checkbox
remains open until the changes are merged.
