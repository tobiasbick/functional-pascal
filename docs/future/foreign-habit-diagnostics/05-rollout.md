# 05 – Rollout

Work packages for the [foreign-habit diagnostics plan](README.md). Each package is one reviewable
change that ends with the `AGENTS.md` definition of done: code, tests, docs, `cargo fmt`,
`cargo build`, `cargo test --workspace`, and `fpas test tests/` where FPAS tests changed.

Packages run in order; each lists its catalog entries from [02-catalog.md](02-catalog.md).

## Package status

| Done | Package | Catalog entries | Depends on | Effort |
| --- | --- | --- | --- | --- |
| [ ] | [P0 Structure](#p0-structure) | – (moves only) | plan approved | small |
| [ ] | [E1 Short experiment](#e1-short-experiment) | new entries | P0 | small |
| [ ] | [P1 Routines and results](#p1-routines-and-results) | H-DECL-07, H-DECL-08, H-CALL-01, H-CALL-02, H-CALL-03 | E1 | medium |
| [ ] | [P2 Declarations](#p2-declarations) | H-DECL-01, H-DECL-02, H-DECL-03, H-DECL-05, H-DECL-06, H-DECL-10 | E1 | medium |
| [ ] | [P3 Control flow](#p3-control-flow) | H-STMT-02, H-STMT-03, H-STMT-04, H-STMT-05, H-STMT-06, H-STMT-07, H-STMT-08 | E1 | medium |
| [ ] | [P4 Tokens and operators](#p4-tokens-and-operators) | H-DECL-04, H-CALL-06, H-EXPR-02, H-EXPR-03, H-EXPR-04, H-LEX-02, H-CALL-04 | E1 | medium |
| [ ] | [P5 Built-in names](#p5-built-in-names) | H-DECL-09, H-CALL-07, H-CALL-08, H-CALL-10, H-CALL-12, H-STMT-09 | E1 | small |
| [ ] | [P6 Experiment and closing](#p6-experiment-and-closing) | new entries from the experiment | P1–P5 | medium |

P1 is the default first package because routine headers and results produce the longest cascades
and occur in almost every program. E1 may reorder P1–P5; they are independent of each other.

## P0 Structure

Refactor-only. No observable behavior change.

- Create `crates/fpas-lexer/src/lexer/foreign/`, `crates/fpas-parser/src/parser/foreign/`,
  `crates/fpas-sema/src/check/foreign/` with `mod.rs` and the shared hint helper from
  [03-architecture.md](03-architecture.md#hint-format).
- Move `crates/fpas-lexer/src/lexer/invalid_comments.rs` to `lexer/foreign/comments.rs`.
- Allocate `LEX_FOREIGN_TOKEN` and `PARSE_FOREIGN_SYNTAX` from
  [03-architecture.md](03-architecture.md#diagnostic-codes); they stay unused until P1 and P4.
- Docs: code catalog entries in `docs/pascal/tools/diagnostics.md` follow with the first package
  that emits each code. Tests: existing suite plus the code-range tests in `fpas-diagnostics`.

## E1 Short experiment

- Run tasks 1–5 of the blind authoring experiment, Delphi style only
  ([01-catalog-method.md](01-catalog-method.md#blind-authoring-experiment)).
- Add new entries to [02-catalog.md](02-catalog.md) and assign them to P1–P5.
- Reorder P1–P5 by how often their habits occurred. Docs-only; no code change.

## P1 Routines and results

- Parser `foreign/routines.rs`: `procedure Q;` / `function F: T;` without parameter list;
  Ada `function F return T is … end F;`; `Result :=` as statement start.
- Sema `foreign/result_assignment.rs`: assignment to the enclosing function's name; `Exit(V)`
  via `foreign/builtins.rs`.
- Recovery: after a missing parameter list, continue as if `()` was written.
- Acceptance: each probe from the catalog yields exactly one diagnostic with the proposed help.

## P2 Declarations

- Parser `foreign/declarations.rs`: `const N = V;`, var sections (report once, at the section),
  `class`, `interface`/`implementation` in a unit.
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

- Lexer `foreign/operators.rs` and `foreign/strings.rs`: `@`, `^`, `&&`, `||`, `!`, `==`, `!=`,
  double-quoted strings, Ada `--` comments.
- Parser `foreign/operators.rs` and `foreign/arguments.rs`: `/=`, `X => V`.
- Recovery: map the foreign token to its FPAS equivalent for the rest of the parse so the
  expression stays usable.

## P5 Built-in names

- Sema `foreign/builtins.rs` table: `Inc`, `Dec`, `Exit`, `nil`, `char`, `Length` (drop the stale
  remark about removed helper units), `IntToStr` member naming.
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
