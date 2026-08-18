# Crate review follow-ups (2026-08)

Intake from a defect-first review of the Rust workspace crates. These are **bugfixes against the current spec**, not new language features.

**Start here if you are implementing:** [`how-to-implement.md`](how-to-implement.md), then **one** file in [`tasks/`](tasks/).

The review notes (do not implement from this list ad hoc) are in [`findings.md`](findings.md).

## Recommended order (cheaper model)

Do tasks in this order. Stop after one task per session unless the user names more.

| Order | Task | Difficulty | Why this slot |
|-------|------|------------|----------------|
| 1 | [01 lexer exponent](tasks/01-lexer-exponent.md) | easy | Isolated, existing numeric tests |
| 2 | [02 Slice/Substring/Delete overflow](tasks/02-std-range-overflow.md) | easy | Local `checked_add` |
| 3 | [03 FromChar / Pad / IntToHex](tasks/03-std-fromchar-pad.md) | easy | Reuse `checked_collection_len` |
| 4 | [04 Window / GotoXY errors](tasks/04-std-console-coords.md) | easy | Spec already requires the errors |
| 5 | [05 fmt `uses` comments](tasks/05-fmt-uses-comments.md) | easy | Comment emit only |
| 6 | [06 JSONL blank lines](tasks/06-debug-jsonl-blank.md) | easy | Skip empty lines |
| 7 | [07 DAP pagination `0`](tasks/07-dap-pagination.md) | easy | Interpret `0` as all |
| 8 | [08 empty parser sections](tasks/08-parser-empty-sections.md) | easy | One-or-more EBNF |
| 9 | [10 keep static AST](tasks/10-parser-static-ast.md) | easy | Stop discarding the tree |
| 10 | [25 POSIX paths on Windows](tasks/25-project-posix-paths.md) | easy | Small path helper |
| 11 | [09 expression recovery](tasks/09-parser-expr-recovery.md) | medium | Sync-token set |
| 12 | [11 enum alias lowering](tasks/11-compiler-enum-alias.md) | medium | Backing value, not index |
| 13 | [12 closure initializers](tasks/12-compiler-closure-init.md) | medium | Discover decls |
| 14 | [15 record literals](tasks/15-sema-record-literals.md) | medium | Do before or with 14 |
| 15 | [14 named record identity](tasks/14-sema-named-records.md) | medium | After 15 if literals break |
| 16 | [16 imported `Color.Red`](tasks/16-sema-imported-enum.md) | medium | Interface install |
| 17 | [17 case exhaustiveness](tasks/17-sema-exhaustiveness.md) | medium | Resolved labels |
| 18 | [18 public + private type](tasks/18-sema-export-private-type.md) | medium | Export walk |
| 19 | [23 library origin `Own`](tasks/23-project-origin.md) | medium | Origin merge |
| 20 | [24 linker layouts](tasks/24-linker-layouts.md) | medium | Compare types |
| 21 | [26 test timeouts](tasks/26-cli-test-timeout.md) | medium | Default + ready-wait |
| 22 | [27 DAP breakpoints](tasks/27-dap-breakpoints.md) | medium | Atomic replace |
| 23 | [28 DAP source paths](tasks/28-dap-source-paths.md) | medium | Case + remap |
| 24 | [29 LSP paths / analysis](tasks/29-lsp-paths-analysis.md) | medium | Three small path bugs |
| 25 | [30 fmt `#` codes + glob symlinks](tasks/30-fmt-char-codes-symlinks.md) | easy/medium | Two independent fixes |
| 26 | [31 leftovers](tasks/31-leftovers.md) | easy | Help flag, tests, line caps |

## Leave for a stronger model (hard)

Read the task anyway; if the first approach fails existing tests, stop and report.

| Task | Why hard |
|------|----------|
| [13 generic `T` vs every type](tasks/13-sema-generic-compat.md) | `compatible_with` wildcard is also used for Map-style function types; do not delete the arm blindly |
| [19 Wait hot-spin on shutdown](tasks/19-vm-wait-shutdown.md) | Task runtime; tests must not hang |
| [20 helped-task panic](tasks/20-vm-help-fail.md) | Depends on 19; pool vs waiter failure paths |
| [21 callback Sleep/Yield](tasks/21-vm-callback-sleep.md) | Nested worker `task_id` |
| [22 IndexSet + Application.Run](tasks/22-vm-indexset-graph.md) | Failure-path state; graph tests may be thin |

## Status

All tasks start **open**. Do not edit these files to mark completion unless the user asks. Implementation lives in the crates, with tests named in each task.

## Out of scope

- Language syntax/semantics changes (see AGENTS.md language gate).
- Package registries, CI workflows, `unsafe`.
- Refactors that are not required to land the fix.
