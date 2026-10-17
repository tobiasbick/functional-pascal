# 05 – Rollout

Work packages for the [foreign-habit diagnostics plan](README.md). Each package is one reviewable
change that ends with the `AGENTS.md` definition of done: code, tests, docs, `cargo fmt`,
`cargo build`, `cargo test --workspace`, and `fpas test tests/` where FPAS tests changed.

Packages run in order; each lists its catalog entries from [02-catalog.md](02-catalog.md).

## Package status

| Done | Package | Catalog entries | Depends on | Effort |
| --- | --- | --- | --- | --- |
| [x] | [P0 Structure](#p0-structure) | – (moves only) | plan approved | small |
| [x] | [E1 Short experiment](#e1-short-experiment) | new entries | P0 | small |
| [x] | [P1 Routines and results](#p1-routines-and-results) | H-DECL-07, H-DECL-08, H-CALL-01, H-CALL-02, H-CALL-03, H-CALL-13 | E1 | medium |
| [x] | [P2 Declarations](#p2-declarations) | H-DECL-01, H-DECL-02, H-DECL-03, H-DECL-05, H-DECL-06, H-DECL-10, H-DECL-11, H-DECL-12, H-DECL-13, H-DECL-14 | E1 | medium |
| [ ] | [P5 Built-in names](#p5-built-in-names) | H-DECL-09, H-DECL-15, H-CALL-07, H-CALL-08, H-CALL-10, H-CALL-12, H-CALL-15, H-CALL-17, H-CALL-19, H-STMT-09 | E1 | small |
| [ ] | [P3 Control flow](#p3-control-flow) | H-STMT-02, H-STMT-03, H-STMT-04, H-STMT-05, H-STMT-06, H-STMT-07, H-STMT-08 | E1 | medium |
| [ ] | [P4 Tokens and operators](#p4-tokens-and-operators) | H-DECL-04, H-CALL-06, H-EXPR-02, H-EXPR-03, H-EXPR-04, H-LEX-02, H-CALL-04, H-CALL-16 | E1 | medium |
| [ ] | [P6 Experiment and closing](#p6-experiment-and-closing) | new entries from the experiment | P1–P5 | medium |

Order after E1: P1, P2, P5, P3, P4 (the table lists them in this order). Packages whose habits
stop the parse come first, because they hide every later habit: P1 for `const` parameters
(H-CALL-13, fatal in three of five E1 programs), P2 for var sections and bounded arrays (seen in all
five). P5 follows because its habits appeared in four programs and each costs one table line.
P3 (three programs) and P4 (two) close. The packages stay independent of each other; see the
[E1 results](02-catalog.md#e1-short-experiment-2026-10-17).

## P0 Structure

**Done.** Refactor-only; no observable behavior change.

- Moved `crates/fpas-lexer/src/lexer/invalid_comments.rs` to `lexer/foreign/comments.rs` under a
  new `lexer/foreign/mod.rs`. Comment diagnostics (FP1013) are unchanged.
- Deferred to the first package that uses them, because an unused module, helper or code would
  be dead code, and the code-catalog test in `fpas-diagnostics` requires a documented example
  for every allocated code:
  - parser and sema `foreign/` modules and the shared hint helper from
    [03-architecture.md](03-architecture.md#hint-format): P1;
  - `PARSE_FOREIGN_SYNTAX`: the first package that emits it (P1 for Ada
    `function F return T is`);
  - `LEX_FOREIGN_TOKEN`: P4.

## E1 Short experiment

**Done** on 2026-10-17; results in [02-catalog.md](02-catalog.md#e1-short-experiment-2026-10-17).

- Run tasks 1–5 of the blind authoring experiment, Delphi style only
  ([01-catalog-method.md](01-catalog-method.md#blind-authoring-experiment)).
- Add new entries to [02-catalog.md](02-catalog.md) and assign them to P1–P5.
- Reorder P1–P5 by how often their habits occurred. Docs-only; no code change.

## P1 Routines and results

**Done** on 2026-10-17.

- Shared helper `fpas_diagnostics::ForeignHabit` builds the help ("… FPAS writes `…`.") and fills
  `found`/`expected` with the foreign text and its replacement, the single point A3 can extend.
- Parser `foreign/routines.rs`: `const` parameters, missing parameter lists, Ada headers (the Ada
  body is skipped through `end Name;`), `Result :=` and `Result.Field :=`. Allocated FP2023
  (`PARSE_FOREIGN_SYNTAX`). A missing parameter list keeps FP2001 with specific help.
- Sema `check/foreign/`: assignment to the enclosing function's name reuses FP3005 (the
  assignment is invalid) instead of the misleading FP3006; `Exit` gets help on FP3003 through
  the `builtins.rs` table that P5 extends.
- Docs: [diagnostics handbook](../../pascal/tools/diagnostics.md#diagnostics-for-other-language-spellings)
  section with the permanent habit table and FP2023 catalog row; three new rows in the
  [agent reference](../../pascal/agent-reference.md#1-translate-habits-before-writing) habit table.
- Tests: `fpas-parser` `tests/errors/foreign_routines.rs` and `fpas-sema`
  `tests/integration/foreign_routines.rs` (positive, negative, recovery).
- Probes: every P1 probe now yields exactly one diagnostic (before: up to 12). The E1 programs
  are no longer stopped by `const` parameters; their first P2 and P3 habits are reported now.

Original scope:

- Create `crates/fpas-parser/src/parser/foreign/` and `crates/fpas-sema/src/check/foreign/`
  with the shared hint helper; allocate and document `PARSE_FOREIGN_SYNTAX` if a P1 entry emits it.
- Parser `foreign/routines.rs`: `procedure Q;` / `function F: T;` without parameter list;
  Ada `function F return T is … end F;`; `Result :=` as statement start.
- Parser `foreign/routines.rs`: Delphi `const` parameter modifier (H-CALL-13); recover by
  skipping the modifier so the rest of the header and body still parse.
- Sema `foreign/result_assignment.rs`: assignment to the enclosing function's name; `Exit(V)`
  via `foreign/builtins.rs`.
- Recovery: after a missing parameter list, continue as if `()` was written.
- Acceptance: each probe from the catalog yields exactly one diagnostic with the proposed help.

## P2 Declarations

**Done** on 2026-10-17.

- Parser `foreign/bindings.rs`: one binding-item parser for all `const`/`var` sites; canonical
  input yields the canonical AST. Pascal sections, grouped names, missing initializers, Pascal
  `=` initializers, bounded arrays with `(…)` constants, Ada bindings without keyword and
  routine declaration sections before `begin` are reported once per group, with the actual
  names, types and initializers in the help. Type sections without keywords are reported once.
- Parser `foreign/classes.rs` (`class`), `foreign/units.rs` (`interface`/`implementation`,
  unit `end.`, Delphi run-time units such as `SysUtils`) and `foreign/generics.rs` (the
  angle-bracket hint, moved from `decl/type_arguments.rs`, now naming `array of T` for `TList`).
  The type-parameter hint `type Box<T>` stays in `decl/data/type_defs.rs`; it is an FPAS rule
  for both routine and type parameters.
- H-DECL-10 had no follow-up errors left on the current tree; only its help was improved.
- Docs: habit table and FP2015 text in the
  [diagnostics handbook](../../pascal/tools/diagnostics.md#diagnostics-for-other-language-spellings),
  four rows in the [agent reference](../../pascal/agent-reference.md#1-translate-habits-before-writing).
- Tests: `fpas-parser` `tests/errors/foreign_declarations.rs`; shared assertions moved to
  `tests/errors/foreign_habit_assertions.rs`; three existing tests updated to one diagnostic per
  group and whole-application spans.
- Probes: every P2 probe yields exactly one diagnostic (before: up to 10). In the E1 programs
  all P2 habits are now single diagnostics; the remaining cascades come from P3 and P4 habits.

Original scope:

- Parser `foreign/declarations.rs`: `const N = V;`, var sections (report once, at the section),
  `class`, `interface`/`implementation` in a unit.
- Grouped variable names, `array[L..H] of T` with `= (…)` initializers, `var X: T;` without
  initializer, Delphi RTL units such as `uses SysUtils;` (H-DECL-11 to H-DECL-14).
- Improve FP2015 help to reuse the actual initializer (H-DECL-03).
- Remove the follow-up errors after `TList<Integer>` (H-DECL-10).
- Move the existing angle-bracket and FP2015 hints into `foreign/declarations.rs`.

## P3 Control flow

- Parser `foreign/control_flow.rs`: `begin … end` used as a branch or loop body; missing `;`
  before `else`; Pascal `for I := …` and Ada `for I in … loop`; `try … except`.
- Move the case-arm `when` hint into `foreign/control_flow.rs`; drop its `end case;` follow-up.
- Pay attention to the `begin … end` ambiguity in
  [04-recovery-and-false-positives.md](04-recovery-and-false-positives.md#ambiguities-to-respect).

## P4 Tokens and operators

- Allocate and document `LEX_FOREIGN_TOKEN`.
- Lexer `foreign/operators.rs` and `foreign/strings.rs`: `@`, `^`, `&&`, `||`, `!`, `==`, `!=`,
  double-quoted strings, Ada `--` comments.
- Parser `foreign/arguments.rs`: `WriteLn(X:0:2)` width specifiers (H-CALL-16).
- Parser `foreign/operators.rs` and `foreign/arguments.rs`: `/=`, `X => V`.
- Recovery: map the foreign token to its FPAS equivalent for the rest of the parse so the
  expression stays usable.

## P5 Built-in names

- Sema `foreign/builtins.rs` table: `Inc`, `Dec`, `Exit`, `nil`, `char`, `Length` (drop the stale
  remark about removed helper units), `IntToStr` member naming.
- Table additions from E1: `Double`/`Single`/`Extended`, `Low`/`High`, `TryStrToInt`, `Write`
  (H-DECL-15, H-CALL-15, H-CALL-17, H-CALL-19).
- Sema `foreign/constructors.rs`: `T.Create(…)` on a record type.
- Unify the existing `native_migration_hint` entry point with the table where they overlap; keep
  FPAS-internal migrations (retired `Std.*` names) where they are.

## P6 Experiment and closing

- Run the full blind authoring experiment from [01-catalog-method.md](01-catalog-method.md).
- Implement the new high-frequency entries as an additional package or defer them in the catalog.
- Move the catalog to its permanent home ([06-docs-and-tests.md](06-docs-and-tests.md#permanent-catalog)),
  mark A2 implemented in [agent-wishes.md](../agent-wishes.md), and remove this directory.

## Decisions

- **Q-4 (decided).** A short experiment (E1) runs before P1; the full experiment runs in P6.
